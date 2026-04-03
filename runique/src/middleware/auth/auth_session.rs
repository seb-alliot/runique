use crate::admin::permissions::{Droit, Groupe, pull_droits_db, pull_groupes_db};
use crate::config::AppSettings;
use crate::context::RequestExtensions;
use crate::middleware::auth::permissions_cache::{
    cache_permissions, evict_permissions, get_permissions,
};
use crate::middleware::session::session_db::RuniqueSessionStore;
use crate::utils::constante::{
    SESSION_ACTIVE_KEY, SESSION_USER_DROITS_KEY, SESSION_USER_GROUPES_KEY, SESSION_USER_ID_KEY,
    SESSION_USER_IS_STAFF_KEY, SESSION_USER_IS_SUPERUSER_KEY, SESSION_USER_USERNAME_KEY,
};
use axum::{
    extract::Request,
    middleware::Next,
    response::{IntoResponse, Redirect, Response},
};
use sea_orm::DatabaseConnection;
use tower_sessions::Session;

// ═══════════════════════════════════════════════════════════════
// CurrentUser — Utilisateur authentifié
// ═══════════════════════════════════════════════════════════════

#[derive(Clone, Debug, serde::Serialize)]
pub struct CurrentUser {
    pub id: i32,
    pub username: String,
    /// Accès au panneau d'administration (lecture / opérations limitées)
    pub is_staff: bool,
    /// Accès complet — bypass toutes les restrictions admin
    pub is_superuser: bool,
    /// Droits directs de l'utilisateur
    pub droits: Vec<Droit>,
    /// Groupes de l'utilisateur (chaque groupe contient ses droits)
    pub groupes: Vec<Groupe>,
}

impl CurrentUser {
    /// Droits effectifs = droits directs + droits hérités des groupes, dédupliqués.
    pub fn droits_effectifs(&self) -> Vec<Droit> {
        let mut droits = self.droits.clone();
        for groupe in &self.groupes {
            droits.extend(groupe.droits.iter().cloned());
        }
        droits.sort();
        droits.dedup();
        droits
    }

    /// Vérifie si l'utilisateur possède un droit spécifique (par nom).
    #[must_use]
    pub fn has_droit(&self, nom: &str) -> bool {
        self.droits_effectifs().iter().any(|d| d.nom == nom)
    }

    /// Vérifie si l'utilisateur possède au moins un des droits fournis.
    #[must_use]
    pub fn has_any_droit(&self, noms: &[&str]) -> bool {
        noms.iter().any(|n| self.has_droit(n))
    }

    /// Vérifie si l'utilisateur peut accéder au panneau d'administration.
    #[must_use]
    pub fn can_access_admin(&self) -> bool {
        self.is_staff || self.is_superuser
    }

    /// Vérifie les droits pour une opération admin.
    /// `is_superuser` bypass toujours.
    #[must_use]
    pub fn can_admin(&self, required: &[&str]) -> bool {
        if self.is_superuser {
            return true;
        }
        self.has_any_droit(required)
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
pub async fn get_user_id(session: &Session) -> Option<i32> {
    session.get::<i32>(SESSION_USER_ID_KEY).await.ok().flatten()
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
pub async fn login(
    session: &Session,
    db: &DatabaseConnection,
    user_id: i32,
    username: &str,
    is_staff: bool,
    is_superuser: bool,
    db_store: Option<&RuniqueSessionStore>,
    exclusive: bool,
) -> Result<(), tower_sessions::session::Error> {
    let droits = pull_droits_db(db, user_id).await;
    let groupes = pull_groupes_db(db, user_id).await;

    // Cache mémoire — point d'accès unique pour load_user_middleware et le point 6
    cache_permissions(user_id, droits.clone(), groupes.clone());

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
    if let Some(user_id) = session.get::<i32>(SESSION_USER_ID_KEY).await.ok().flatten() {
        evict_permissions(user_id);
    }

    session.remove::<i32>(SESSION_USER_ID_KEY).await?;
    session.remove::<String>(SESSION_USER_USERNAME_KEY).await?;
    session.remove::<bool>(SESSION_USER_IS_STAFF_KEY).await?;
    session
        .remove::<bool>(SESSION_USER_IS_SUPERUSER_KEY)
        .await?;
    session
        .remove::<Vec<Droit>>(SESSION_USER_DROITS_KEY)
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

/// Vérifie si l'utilisateur a un droit donné.
pub async fn has_permission(session: &Session, permission: &str) -> bool {
    let droits = session
        .get::<Vec<Droit>>(SESSION_USER_DROITS_KEY)
        .await
        .ok()
        .flatten()
        .unwrap_or_default();

    let groupes = session
        .get::<Vec<Groupe>>(SESSION_USER_GROUPES_KEY)
        .await
        .ok()
        .flatten()
        .unwrap_or_default();

    let is_superuser = session
        .get::<bool>(SESSION_USER_IS_SUPERUSER_KEY)
        .await
        .ok()
        .flatten()
        .unwrap_or(false);

    if is_superuser {
        return true;
    }

    let mut all_droits = droits;
    for g in groupes {
        all_droits.extend(g.droits);
    }
    all_droits.iter().any(|d| d.nom == permission)
}

// ═══════════════════════════════════════════════════════════════
// Middlewares Axum
// ═══════════════════════════════════════════════════════════════

/// Middleware : protège les routes (authentification requise).
pub async fn login_required(session: Session, request: Request, next: Next) -> Response {
    if is_authenticated(&session).await {
        next.run(request).await
    } else {
        let redirect_anonymous = AppSettings::default().redirect_anonymous;
        Redirect::to(&redirect_anonymous).into_response()
    }
}

/// Middleware : redirige les utilisateurs déjà connectés.
pub async fn redirect_if_authenticated(session: Session, request: Request, next: Next) -> Response {
    if is_authenticated(&session).await {
        let redirect_url = AppSettings::default().user_connected;
        Redirect::to(&redirect_url).into_response()
    } else {
        next.run(request).await
    }
}

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

        // Droits et groupes depuis le cache mémoire (plus rapide que la session)
        let (droits, groupes) = match get_permissions(user_id) {
            Some(cached) => (cached.droits.clone(), cached.groupes.clone()),
            None => (vec![], vec![]),
        };

        let current_user = CurrentUser {
            id: user_id,
            username,
            is_staff,
            is_superuser,
            droits,
            groupes,
        };

        let extensions = RequestExtensions::new().with_current_user(current_user);
        extensions.inject_request(&mut request);
    } else {
        let _ = session.delete().await;
    }

    next.run(request).await
}
