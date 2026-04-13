//! Middlewares Axum pour l'espace admin — redirection si non authentifié, vérification des permissions.
use axum::{
    extract::Request,
    middleware::Next,
    response::{IntoResponse, Redirect, Response},
};
use tower_sessions::Session;

use crate::auth::session::is_admin_authenticated;

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
pub(crate) async fn admin_required(session: Session, request: Request, next: Next) -> Response {
    if !is_admin_authenticated(&session).await {
        return Redirect::to("/").into_response();
    }
    next.run(request).await
}
