//! Axum middlewares for the admin space — redirection if not authenticated, permission verification.
use axum::{
    extract::Request,
    middleware::Next,
    response::{IntoResponse, Redirect, Response},
};
use tower_sessions::Session;

use crate::auth::session::is_admin_authenticated;

/// Middleware: admin access required (is_staff OR is_superuser)
///
/// Redirects to / if not authenticated.
/// Returns 403 if authenticated but without admin rights.
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
