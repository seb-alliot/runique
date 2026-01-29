// runique/src/middleware/auth.rs

use crate::config::AppSettings;
use crate::context::RequestExtensions;
use crate::utils::constante::{SESSION_USER_ID_KEY, SESSION_USER_USERNAME_KEY};
use axum::{
    extract::Request,
    middleware::Next,
    response::{IntoResponse, Redirect, Response},
};
use tower_sessions::Session;

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

/// Connecte un utilisateur (stocke son ID et username en session)
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

/// Déconnecte un utilisateur (supprime la session)
pub async fn logout(session: &Session) -> Result<(), tower_sessions::session::Error> {
    session.delete().await
}

/// Middleware pour protéger les routes (nécessite authentification)
/// ...
/// # Exemple
/// ```rust
/// # use axum::{Router, routing::get};
/// # async fn dashboard() -> &'static str { "Dashboard" }
/// # async fn profile() -> &'static str { "Profile" }
/// use runique::middleware::auth::login_required;
///
/// // Utilisation de Router<()> pour aider l'inférence de type
/// let protected_routes: Router = Router::new()
///     .route("/dashboard", get(dashboard))
///     .route("/profile", get(profile))
///     .layer(axum::middleware::from_fn(login_required));
/// ```
pub async fn login_required(session: Session, request: Request, next: Next) -> Response {
    // Vérifier si l'utilisateur est authentifié
    if is_authenticated(&session).await {
        next.run(request).await
    } else {
        // Rediriger vers la page de login
        let redirect_anonymous = AppSettings::default().redirect_anonymous;
        Redirect::to(&redirect_anonymous).into_response()
    }
}

/// Middleware pour rediriger les utilisateurs connectés
/// (utile pour les pages login/register)
///
/// # Exemple
/// ```rust
/// # use axum::{Router, routing::get};
/// # async fn login_page() -> &'static str { "Login" }
/// # async fn register_page() -> &'static str { "Register" }
/// use runique::middleware::auth::redirect_if_authenticated;
///
/// let public_routes: Router = Router::new()
///     .route("/login", get(login_page))
///     .route("/register", get(register_page))
///     .layer(axum::middleware::from_fn(redirect_if_authenticated));
/// ```
pub async fn redirect_if_authenticated(session: Session, request: Request, next: Next) -> Response {
    // Si déjà connecté, rediriger vers dashboard
    if is_authenticated(&session).await {
        let redirect_url = AppSettings::default().user_connected;
        Redirect::to(&redirect_url).into_response()
    } else {
        next.run(request).await
    }
}

/// Middleware optionnel : charge les infos utilisateur dans les extensions
/// (permet d'accéder à l'utilisateur dans les handlers sans session)
///
/// # Exemple
/// ```rust
/// # use axum::{Router, routing::get};
/// # async fn dashboard() -> &'static str { "Dashboard" }
/// use runique::middleware::auth::{load_user_middleware, CurrentUser};
///
/// let app: Router = Router::new()
///     .route("/dashboard", get(dashboard))
///     .layer(axum::middleware::from_fn(load_user_middleware));
/// ```
#[derive(Clone, Debug)]
pub struct CurrentUser {
    pub id: i32,
    pub username: String,
}

pub async fn load_user_middleware(session: Session, mut request: Request, next: Next) -> Response {
    // Charger l'utilisateur si authentifié
    if let (Some(user_id), Some(username)) =
        (get_user_id(&session).await, get_username(&session).await)
    {
        let current_user = CurrentUser {
            id: user_id,
            username,
        };

        // Injection via la structure centralisée
        let extensions = RequestExtensions::new().with_current_user(current_user);

        extensions.inject_request(&mut request);
    }

    next.run(request).await
}

/// Helper pour vérifier les permissions (exemple)
///
/// # Note
/// Cette fonction est un stub de base. Pour une implémentation complète,
/// vous devrez récupérer les rôles/permissions depuis la base de données.
///
/// # Exemple d'implémentation complète
/// ```rust,no_run
/// # use tower_sessions::Session;
/// # use runique::middleware::auth::get_user_id;
/// pub async fn has_permission(session: &Session, permission: &str) -> bool {
///     if let Some(user_id) = get_user_id(session).await {
///         // Récupérer les permissions depuis la DB
///         true
///     } else {
///         false
///     }
/// }
/// ```
pub async fn has_permission(session: &Session, _permission: &str) -> bool {
    // Stub de base : pour l'instant, seul l'authentification est vérifiée
    // TODO: Implémenter la logique complète de permissions avec la DB
    is_authenticated(session).await
}
