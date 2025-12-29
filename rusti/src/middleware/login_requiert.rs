// rusti/src/middleware/auth.rs

use axum::{
    middleware::Next,
    response::{IntoResponse, Redirect, Response},
    extract::Request,
};
use tower_sessions::Session;

/// Clé de session pour stocker l'ID utilisateur
pub const SESSION_USER_ID_KEY: &str = "_user_id";
pub const SESSION_USER_USERNAME_KEY: &str = "_username";

/// Vérifie si l'utilisateur est authentifié
pub async fn is_authenticated(session: &Session) -> bool {
    session.get::<i32>(SESSION_USER_ID_KEY)
        .await
        .ok()
        .flatten()
        .is_some()
}

/// Récupère l'ID de l'utilisateur connecté
pub async fn get_user_id(session: &Session) -> Option<i32> {
    session.get::<i32>(SESSION_USER_ID_KEY)
        .await
        .ok()
        .flatten()
}

/// Récupère le username de l'utilisateur connecté
pub async fn get_username(session: &Session) -> Option<String> {
    session.get::<String>(SESSION_USER_USERNAME_KEY)
        .await
        .ok()
        .flatten()
}

/// Connecte un utilisateur (stocke son ID et username en session)
pub async fn login_user(session: &Session, user_id: i32, username: &str) -> Result<(), tower_sessions::session::Error> {
    session.insert(SESSION_USER_ID_KEY, user_id).await?;
    session.insert(SESSION_USER_USERNAME_KEY, username.to_string()).await?;
    Ok(())
}

/// Déconnecte un utilisateur (supprime la session)
pub async fn logout_user(session: &Session) -> Result<(), tower_sessions::session::Error> {
    session.delete().await
}

/// Middleware pour protéger les routes (nécessite authentification)
///
/// # Exemple
///
/// ```rust
/// use rusti::middleware::auth::login_required;
/// use axum::routing::get;
///
/// let protected_routes = Router::new()
///     .route("/dashboard", get(dashboard))
///     .route("/profile", get(profile))
///     .layer(axum::middleware::from_fn(login_required));
/// ```
pub async fn login_required(
    session: Session,
    request: Request,
    next: Next,
) -> Response {
    // Vérifier si l'utilisateur est authentifié
    if is_authenticated(&session).await {
        next.run(request).await
    } else {
        // Rediriger vers la page de login
        let login_url = "/login"; // Configurable via settings ?
        Redirect::to(login_url).into_response()
    }
}

/// Middleware pour rediriger les utilisateurs connectés
/// (utile pour les pages login/register)
///
/// # Exemple
///
/// ```rust
/// use rusti::middleware::auth::redirect_if_authenticated;
///
/// let public_routes = Router::new()
///     .route("/login", get(login_page))
///     .route("/register", get(register_page))
///     .layer(axum::middleware::from_fn(redirect_if_authenticated));
/// ```
pub async fn redirect_if_authenticated(
    session: Session,
    request: Request,
    next: Next,
) -> Response {
    // Si déjà connecté, rediriger vers dashboard
    if is_authenticated(&session).await {
        let redirect_url = "/dashboard"; // Configurable ?
        Redirect::to(redirect_url).into_response()
    } else {
        next.run(request).await
    }
}

/// Middleware optionnel : charge les infos utilisateur dans les extensions
/// (permet d'accéder à l'utilisateur dans les handlers sans session)
///
/// # Exemple
///
/// ```rust
/// use rusti::middleware::auth::{load_user_middleware, CurrentUser};
/// use axum::Extension;
///
/// async fn dashboard(Extension(user): Extension<CurrentUser>) -> String {
///     format!("Hello, {}!", user.username)
/// }
///
/// let app = Router::new()
///     .route("/dashboard", get(dashboard))
///     .layer(axum::middleware::from_fn(load_user_middleware));
/// ```
#[derive(Clone, Debug)]
pub struct CurrentUser {
    pub id: i32,
    pub username: String,
}

pub async fn load_user_middleware(
    session: Session,
    mut request: Request,
    next: Next,
) -> Response {
    // Charger l'utilisateur si authentifié
    if let (Some(user_id), Some(username)) = (
        get_user_id(&session).await,
        get_username(&session).await,
    ) {
        let current_user = CurrentUser { id: user_id, username };
        request.extensions_mut().insert(current_user);
    }

    next.run(request).await
}

/// Helper pour vérifier les permissions (exemple)
pub async fn has_permission(session: &Session, permission: &str) -> bool {
    // TODO: Implémenter la logique de permissions
    // Par exemple, récupérer les rôles depuis la DB
    is_authenticated(session).await
}