use axum::{
    extract::Request,
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Redirect, Response},
};
use tower_sessions::Session;

use crate::middleware::auth::{is_authenticated, CurrentUser};
use crate::utils::constante::SESSION_USER_IS_STAFF_KEY;
use crate::utils::constante::SESSION_USER_IS_SUPERUSER_KEY;

/// Middleware : accès admin requis (is_staff OU is_superuser)
///
/// Redirige vers /admin/login si non authentifié.
/// Retourne 403 si authentifié mais sans droits admin.
///
/// ```rust,ignore
/// Router::new()
///     .nest("/admin", admin_routes)
///     .layer(axum::middleware::from_fn(admin_required))
/// ```
pub async fn admin_required(session: Session, request: Request, next: Next) -> Response {
    // 1. Pas authentifié → login admin
    if !is_authenticated(&session).await {
        return Redirect::to("/admin/login").into_response();
    }

    // 2. Authentifié mais pas staff/superuser → 403
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

    if !is_staff && !is_superuser {
        return (StatusCode::FORBIDDEN, "Accès réservé aux administrateurs").into_response();
    }

    next.run(request).await
}

/// Vérifie les droits d'un `CurrentUser` pour une opération sur une ressource
///
/// Utilisé dans les handlers générés pour valider les permissions granulaires.
///
/// `is_superuser` bypass toujours → retourne `true`
///
/// ```rust,ignore
/// pub async fn admin_users_delete(
///     Extension(user): Extension<CurrentUser>,
/// ) -> Response {
///     if !check_permission(&user, &["admin"]) {
///         return (StatusCode::FORBIDDEN, "Droits insuffisants").into_response();
///     }
///     // ... logique suppression
/// }
/// ```
pub fn check_permission(user: &CurrentUser, required_roles: &[&str]) -> bool {
    user.can_admin(required_roles)
}
