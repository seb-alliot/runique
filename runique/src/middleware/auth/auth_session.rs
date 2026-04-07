//! Middleware de session utilisateur : charge `CurrentUser` depuis la session et injecte dans les extensions.
use crate::admin::permissions::{Groupe, Permission, pull_groupes_db};
use crate::context::RequestExtensions;
use crate::middleware::auth::default_auth::UserEntity;
use crate::middleware::auth::permissions_cache::{
    cache_permissions, evict_permissions, get_permissions,
};
use crate::middleware::auth::user::BuiltinUserEntity;
use crate::middleware::auth::user_trait::RuniqueUser;
use crate::middleware::session::session_db::RuniqueSessionStore;
use crate::utils::constante::{
    SESSION_ACTIVE_KEY, SESSION_USER_GROUPES_KEY, SESSION_USER_ID_KEY, SESSION_USER_IS_STAFF_KEY,
    SESSION_USER_IS_SUPERUSER_KEY, SESSION_USER_USERNAME_KEY,
};
use crate::utils::pk::Pk;
use axum::{extract::Request, middleware::Next, response::Response};
use sea_orm::DatabaseConnection;
use tower_sessions::Session;

/// Utilisateur authentifié injecté dans les extensions de requête.
#[derive(Clone, Debug, serde::Serialize)]
pub struct CurrentUser {
    pub id: Pk,
    pub username: String,
    /// Accès au panneau d'administration (lecture / opérations limitées)
    pub is_staff: bool,
    /// Accès complet — bypass toutes les restrictions admin
    pub is_superuser: bool,
    /// Groupes de l'utilisateur (chaque groupe contient ses permissions)
    pub groupes: Vec<Groupe>,
}

impl CurrentUser {
    /// Retourne les permissions CRUD effectives (Union logique de tous les groupes).
    pub fn permissions_effectives(&self) -> Vec<Permission> {
        let mut agg: std::collections::HashMap<String, Permission> =
            std::collections::HashMap::new();

        for groupe in &self.groupes {
            for perm in &groupe.permissions {
                let entry = agg
                    .entry(perm.resource_key.clone())
                    .or_insert_with(|| Permission {
                        id: 0,
                        resource_key: perm.resource_key.clone(),
                        can_create: false,
                        can_read: false,
                        can_update: false,
                        can_delete: false,
                        can_update_own: false,
                        can_delete_own: false,
                    });
                entry.can_create |= perm.can_create;
                entry.can_read |= perm.can_read;
                entry.can_update |= perm.can_update;
                entry.can_delete |= perm.can_delete;
                entry.can_update_own |= perm.can_update_own;
                entry.can_delete_own |= perm.can_delete_own;
            }
        }

        agg.into_values().collect()
    }

    /// Vérifie si l'utilisateur possède une permission globale stricte (peut être affiné).
    #[must_use]
    pub fn can_access_resource(&self, resource_key: &str) -> bool {
        if self.is_superuser {
            return true;
        }
        self.permissions_effectives()
            .iter()
            .any(|d| d.resource_key == resource_key && d.can_read)
    }

    /// Vérifie si l'utilisateur peut accéder au panneau d'administration.
    #[must_use]
    pub fn can_access_admin(&self) -> bool {
        self.is_staff || self.is_superuser
    }
}

// ═══════════════════════════════════════════════════════════════
// Helpers de session
// ═══════════════════════════════════════════════════════════════

/// Vérifie si l'utilisateur est authentifié.
pub async fn is_authenticated(session: &Session) -> bool {
    session
        .get::<i32>(SESSION_USER_ID_KEY)
        .await
        .ok()
        .flatten()
        .is_some()
}

/// Vérifie si l'utilisateur est authentifié et a accès à l'admin.
pub async fn is_admin_authenticated(session: &Session) -> bool {
    if !is_authenticated(session).await {
        return false;
    }

    let is_staff = session
        .get::<bool>(SESSION_USER_IS_STAFF_KEY)
        .await
        .ok()
        .flatten()
        .unwrap_or(false);

    let is_superuser = session
        .get::<bool>(SESSION_USER_IS_SUPERUSER_KEY)
        .await
        .ok()
        .flatten()
        .unwrap_or(false);

    is_staff || is_superuser
}

/// Récupère l'ID de l'utilisateur connecté.
pub async fn get_user_id(session: &Session) -> Option<Pk> {
    session.get::<Pk>(SESSION_USER_ID_KEY).await.ok().flatten()
}

/// Récupère le username de l'utilisateur connecté.
pub async fn get_username(session: &Session) -> Option<String> {
    session
        .get::<String>(SESSION_USER_USERNAME_KEY)
        .await
        .ok()
        .flatten()
}

// ═══════════════════════════════════════════════════════════════
// Login unifié
// ═══════════════════════════════════════════════════════════════

/// Connecte un utilisateur — charge ses droits et groupes depuis la DB.
///
/// Si `db_store` est fourni, persiste la session en DB (multi-appareils).
/// Si `exclusive` est `true`, invalide les autres sessions de l'utilisateur.
///
/// ```rust,ignore
/// login(&session, &db, user.id, &user.username, user.is_staff, user.is_superuser, None, false).await?;
/// ```
#[allow(clippy::too_many_arguments)]
pub async fn login(
    session: &Session,
    db: &DatabaseConnection,
    user_id: Pk,
    username: &str,
    is_staff: bool,
    is_superuser: bool,
    db_store: Option<&RuniqueSessionStore>,
    exclusive: bool,
) -> Result<(), tower_sessions::session::Error> {
    let groupes = pull_groupes_db(db, user_id).await;

    // Cache mémoire — point d'accès unique pour load_user_middleware et le point 6
    cache_permissions(user_id, groupes.clone());

    session.insert(SESSION_USER_ID_KEY, user_id).await?;
    session
        .insert(SESSION_USER_USERNAME_KEY, username.to_string())
        .await?;
    session.insert(SESSION_USER_IS_STAFF_KEY, is_staff).await?;
    session
        .insert(SESSION_USER_IS_SUPERUSER_KEY, is_superuser)
        .await?;

    // Persistance DB
    if let Some(store) = db_store {
        let cookie_id = session.id().map(|id| id.to_string()).unwrap_or_default();
        let session_id = uuid::Uuid::new_v4().to_string();
        let expires_at = chrono::Utc::now().naive_utc() + chrono::Duration::hours(24);

        let _ = store
            .create(&cookie_id, user_id, &session_id, expires_at)
            .await;

        if exclusive {
            let _ = store.invalidate_other_sessions(user_id, &cookie_id).await;
        }
    }

    Ok(())
}

/// Connecte un utilisateur à partir de son seul `user_id` — charge les données depuis la DB.
///
/// Raccourci générique pour tout flux d'authentification (inscription, OAuth, magic link…)
/// qui dispose déjà de l'identifiant utilisateur sans avoir besoin de repasser les champs.
///
/// Retourne `Ok(())` sans créer de session si le compte est inactif (`is_active = false`).
///
/// Utilise [`BuiltinUserEntity`] pour la recherche. Pour un modèle custom, utiliser [`login`] directement.
pub async fn auth_login(
    session: &Session,
    db: &DatabaseConnection,
    user_id: Pk,
) -> Result<(), tower_sessions::session::Error> {
    let Some(user) = BuiltinUserEntity::find_by_id(db, user_id).await else {
        return Ok(());
    };
    if !user.is_active() {
        return Ok(());
    }
    login(
        session,
        db,
        user.user_id(),
        user.username(),
        user.is_staff(),
        user.is_superuser(),
        None,
        false,
    )
    .await
}

/// Déconnecte un utilisateur — supprime la session mémoire et l'entrée DB si fournie.
pub async fn logout(
    session: &Session,
    db_store: Option<&RuniqueSessionStore>,
) -> Result<(), tower_sessions::session::Error> {
    // Suppression DB avant de vider la session (cookie_id encore accessible)
    if let Some(store) = db_store {
        let cookie_id = session.id().map(|id| id.to_string()).unwrap_or_default();
        let _ = store.delete(&cookie_id).await;
    }

    // Vider le cache permissions
    if let Some(user_id) = session.get::<Pk>(SESSION_USER_ID_KEY).await.ok().flatten() {
        evict_permissions(user_id);
    }

    session.remove::<i32>(SESSION_USER_ID_KEY).await?;
    session.remove::<String>(SESSION_USER_USERNAME_KEY).await?;
    session.remove::<bool>(SESSION_USER_IS_STAFF_KEY).await?;
    session
        .remove::<bool>(SESSION_USER_IS_SUPERUSER_KEY)
        .await?;
    session
        .remove::<Vec<Groupe>>(SESSION_USER_GROUPES_KEY)
        .await?;
    session.remove::<i64>(SESSION_ACTIVE_KEY).await?;
    session.delete().await
}

/// Protège une session anonyme contre le cleanup.
pub async fn protect_session(
    session: &Session,
    duration_secs: i64,
) -> Result<(), tower_sessions::session::Error> {
    let protect_until = chrono::Utc::now().timestamp() + duration_secs;
    session.insert(SESSION_ACTIVE_KEY, protect_until).await
}

/// Retire la protection manuelle d'une session anonyme.
pub async fn unprotect_session(session: &Session) -> Result<(), tower_sessions::session::Error> {
    session.remove::<i64>(SESSION_ACTIVE_KEY).await?;
    Ok(())
}

/// Obsolète : remaniement matriciel
pub async fn has_permission(_session: &Session, _permission: &str) -> bool {
    false // Sera remplacé par les vérifs directes de matrice
}

// ═══════════════════════════════════════════════════════════════
// Middlewares Axum
// ═══════════════════════════════════════════════════════════════

/// Middleware : charge les infos utilisateur dans les extensions de la requête.
pub async fn load_user_middleware(session: Session, mut request: Request, next: Next) -> Response {
    if let (Some(user_id), Some(username)) =
        (get_user_id(&session).await, get_username(&session).await)
    {
        let is_staff = session
            .get::<bool>(SESSION_USER_IS_STAFF_KEY)
            .await
            .ok()
            .flatten()
            .unwrap_or(false);

        let is_superuser = session
            .get::<bool>(SESSION_USER_IS_SUPERUSER_KEY)
            .await
            .ok()
            .flatten()
            .unwrap_or(false);

        // Groupes et permissions depuis le cache mémoire (plus rapide que la session)
        let groupes = match get_permissions(user_id) {
            Some(cached) => cached.groupes.clone(),
            None => vec![],
        };

        let current_user = CurrentUser {
            id: user_id,
            username,
            is_staff,
            is_superuser,
            groupes,
        };

        let extensions = RequestExtensions::new().with_current_user(current_user);
        extensions.inject_request(&mut request);
    } else if session.id().is_some() {
        let _ = session.delete().await;
    }

    next.run(request).await
}
