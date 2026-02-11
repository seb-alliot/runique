use crate::config::AppSettings;
use crate::context::RequestExtensions;
use crate::utils::constante::{
    SESSION_USER_ID_KEY, SESSION_USER_IS_STAFF_KEY, SESSION_USER_IS_SUPERUSER_KEY,
    SESSION_USER_ROLES_KEY, SESSION_USER_USERNAME_KEY,
};
use axum::{
    extract::Request,
    middleware::Next,
    response::{IntoResponse, Redirect, Response},
};
use tower_sessions::Session;

// ═══════════════════════════════════════════════════════════════
// CurrentUser — Utilisateur authentifié
// ═══════════════════════════════════════════════════════════════
//
// Loaded from the session via `load_user_middleware`.
// Injected into the request extensions to be
// accessible in all handlers.
//
// Les champs is_staff / is_superuser / roles sont stockés
// en session lors du login (via login_user_full) et relus
// à chaque requête.
// ═══════════════════════════════════════════════════════════════

#[derive(Clone, Debug, serde::Serialize)]
pub struct CurrentUser {
    pub id: i32,
    pub username: String,
    /// Accès au panneau d'administration (lecture / opérations limitées)
    pub is_staff: bool,

    /// Accès complet — bypass toutes les restrictions admin
    pub is_superuser: bool,

    /// Rôles personnalisés (ex: ["editor", "moderator"])
    pub roles: Vec<String>,
}

impl CurrentUser {
    /// Vérifie si l'utilisateur possède un rôle spécifique
    pub fn has_role(&self, role: &str) -> bool {
        self.roles.iter().any(|r| r == role)
    }

    /// Vérifie si l'utilisateur possède au moins un des rôles fournis
    pub fn has_any_role(&self, roles: &[&str]) -> bool {
        roles.iter().any(|role| self.has_role(role))
    }

    /// Vérifie si l'utilisateur peut accéder au panneau d'administration
    ///
    /// `is_superuser` → accès total
    /// `is_staff` → accès limité selon les permissions de chaque ressource
    pub fn can_access_admin(&self) -> bool {
        self.is_staff || self.is_superuser
    }

    /// Vérifie si l'utilisateur est autorisé pour une opération admin donnée
    ///
    /// `is_superuser` bypass toujours → retourne `true`
    /// Sinon vérifie les rôles requis
    pub fn can_admin(&self, required_roles: &[&str]) -> bool {
        if self.is_superuser {
            return true;
        }
        self.has_any_role(required_roles)
    }
}

// ═══════════════════════════════════════════════════════════════
// Helpers de session
// ═══════════════════════════════════════════════════════════════

/// Vérifie si l'utilisateur est authentifié
pub async fn is_authenticated(session: &Session) -> bool {
    session
        .get::<i32>(SESSION_USER_ID_KEY)
        .await
        .ok()
        .flatten()
        .is_some()
}

/// Récupère l'ID de l'utilisateur connecté
pub async fn get_user_id(session: &Session) -> Option<i32> {
    session.get::<i32>(SESSION_USER_ID_KEY).await.ok().flatten()
}

/// Récupère le username de l'utilisateur connecté
pub async fn get_username(session: &Session) -> Option<String> {
    session
        .get::<String>(SESSION_USER_USERNAME_KEY)
        .await
        .ok()
        .flatten()
}

/// Connecte un utilisateur (stocke id + username en session)
///
/// Version basique — sans rôles ni flags admin.
pub async fn login_user(
    session: &Session,
    user_id: i32,
    username: &str,
) -> Result<(), tower_sessions::session::Error> {
    session.insert(SESSION_USER_ID_KEY, user_id).await?;
    session
        .insert(SESSION_USER_USERNAME_KEY, username.to_string())
        .await?;
    Ok(())
}

/// Connecte un utilisateur avec tous ses attributs
///
/// Version complète — inclut is_staff, is_superuser et rôles.
/// À utiliser pour les applications utilisant l'AdminPanel.
pub async fn login_user_full(
    session: &Session,
    user_id: i32,
    username: &str,
    is_staff: bool,
    is_superuser: bool,
    roles: Vec<String>,
) -> Result<(), tower_sessions::session::Error> {
    session.insert(SESSION_USER_ID_KEY, user_id).await?;
    session
        .insert(SESSION_USER_USERNAME_KEY, username.to_string())
        .await?;
    session.insert(SESSION_USER_IS_STAFF_KEY, is_staff).await?;
    session
        .insert(SESSION_USER_IS_SUPERUSER_KEY, is_superuser)
        .await?;
    session.insert(SESSION_USER_ROLES_KEY, roles).await?;
    Ok(())
}

/// Déconnecte un utilisateur (supprime la session)
pub async fn logout(session: &Session) -> Result<(), tower_sessions::session::Error> {
    session.delete().await
}

/// Vérifie si l'utilisateur a une permission donnée
///
/// Implémentation complète à brancher sur la DB selon le modèle du projet.
pub async fn has_permission(session: &Session, _permission: &str) -> bool {
    is_authenticated(session).await
}

// ═══════════════════════════════════════════════════════════════
// Middlewares Axum
// ═══════════════════════════════════════════════════════════════

/// Middleware : protège les routes (authentification requise)
///
/// ```rust,ignore
/// let protected = Router::new()
///     .route("/dashboard", get(dashboard))
///     .layer(axum::middleware::from_fn(login_required));
/// ```
pub async fn login_required(session: Session, request: Request, next: Next) -> Response {
    if is_authenticated(&session).await {
        next.run(request).await
    } else {
        let redirect_anonymous = AppSettings::default().redirect_anonymous;
        Redirect::to(&redirect_anonymous).into_response()
    }
}

/// Middleware : redirige les utilisateurs déjà connectés
///
/// Utile pour les pages login / register.
///
/// ```rust,ignore
/// let public = Router::new()
///     .route("/login", get(login_page))
///     .layer(axum::middleware::from_fn(redirect_if_authenticated));
/// ```
pub async fn redirect_if_authenticated(session: Session, request: Request, next: Next) -> Response {
    if is_authenticated(&session).await {
        let redirect_url = AppSettings::default().user_connected;
        Redirect::to(&redirect_url).into_response()
    } else {
        next.run(request).await
    }
}

/// Middleware : charge les infos utilisateur dans les extensions
///
/// Charge id, username, is_staff, is_superuser et roles depuis la session.
/// Injecte un `CurrentUser` dans les extensions de la requête.
///
/// ```rust,ignore
/// let app = Router::new()
///     .route("/dashboard", get(dashboard))
///     .layer(axum::middleware::from_fn(load_user_middleware));
/// ```
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

        let roles = session
            .get::<Vec<String>>(SESSION_USER_ROLES_KEY)
            .await
            .ok()
            .flatten()
            .unwrap_or_default();

        let current_user = CurrentUser {
            id: user_id,
            username,
            is_staff,
            is_superuser,
            roles,
        };

        let extensions = RequestExtensions::new().with_current_user(current_user);
        extensions.inject_request(&mut request);
    }

    next.run(request).await
}
