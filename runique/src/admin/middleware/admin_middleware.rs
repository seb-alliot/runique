//! Axum middlewares for the admin space — redirection if not authenticated, permission verification.
use std::sync::Arc;

use axum::{
    Extension,
    extract::{MatchedPath, Request},
    middleware::Next,
    response::{IntoResponse, Redirect, Response},
};
use tower_sessions::Session;

use crate::admin::router::admin_router::AdminState;
use crate::auth::session::is_admin_authenticated;

/// Middleware: admin access required (is_staff OR is_superuser)
///
/// - No matched route → 404 (passes through, Axum handles it)
/// - Unauthenticated → redirect to `{prefix}/login`
pub(crate) async fn admin_required(
    Extension(admin): Extension<Arc<AdminState>>,
    session: Session,
    request: Request,
    next: Next,
) -> Response {
    if request.extensions().get::<MatchedPath>().is_none() {
        return next.run(request).await;
    }
    if !is_admin_authenticated(&session).await {
        let login_url = format!("{}/login", admin.config.prefix.trim_end_matches('/'));
        return Redirect::to(&login_url).into_response();
    }
    next.run(request).await
}
