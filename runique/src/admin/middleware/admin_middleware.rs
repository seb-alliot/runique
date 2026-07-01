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
    let authed = is_admin_authenticated(&session).await;
    // Trace the admin access gate (grade check: is_staff || is_superuser), so the
    // most fundamental auth decision — "may this user enter the admin at all" — is
    // visible even on routes that don't run a per-resource access check (dashboard,
    // history). `admin.auth` channel.
    if let Some(level) = crate::utils::runique_log::get_log()
        .admin
        .as_ref()
        .and_then(|a| a.auth)
    {
        let cu = request
            .extensions()
            .get::<crate::auth::session::CurrentUser>();
        let path = request
            .extensions()
            .get::<MatchedPath>()
            .map_or("", MatchedPath::as_str);
        crate::runique_log!(
            level,
            path = %path,
            user = cu.map_or("-", |u| u.username.as_str()),
            is_staff = cu.is_some_and(|u| u.is_staff),
            is_superuser = cu.is_some_and(|u| u.is_superuser),
            granted = authed,
            "admin access gate"
        );
    }
    if !authed {
        let login_url = format!("{}/login", admin.config.prefix.trim_end_matches('/'));
        return Redirect::to(&login_url).into_response();
    }
    next.run(request).await
}
