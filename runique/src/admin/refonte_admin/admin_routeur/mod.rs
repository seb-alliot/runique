//! The rebuilt admin's routes, built apart from the current admin's router.
//!
//! Mounted under `{admin_path}/next/` while both admins coexist, so neither
//! route set collides with the other's `{resource}/{action}`. Only the routes
//! live here: authentication, the current user, `noindex` and `AdminState`
//! come from the layers `build_admin_router` wraps around them, never copied.
pub mod detail;
pub mod list;

use crate::admin::refonte_admin::builder::registered::AdminError;
use crate::admin::refonte_admin::builder::site::AdminSite;
use crate::admin::router::admin_router::AdminState;
use crate::context::template::{AppError, Request};
use crate::errors::error::ErrorContext;
use crate::utils::aliases::AppResult;
use axum::response::{IntoResponse, Redirect, Response};
use axum::{Extension, Json, Router, routing::get};
use std::sync::Arc;

/// The rebuilt admin's routes under `admin_path`, sharing `site`.
pub(crate) fn refonte_router(admin_path: &str, site: AdminSite) -> Router {
    Router::new()
        .route(
            &format!("{admin_path}/next/{{resource}}/list"),
            get(list::list_view),
        )
        .route(
            &format!("{admin_path}/next/{{resource}}/{{id}}/detail"),
            get(detail::detail_view),
        )
        .layer(Extension(Arc::new(site)))
}

/// One answer per outcome, the same for every operation: the value as JSON
/// (until the templates are rebuilt), a refusal as a flash message and a
/// redirect to the dashboard, a database failure as an error page.
async fn respond<T: serde::Serialize>(
    result: Result<T, AdminError>,
    req: &Request,
    admin: &AdminState,
) -> AppResult<Response> {
    match result {
        Ok(value) => Ok(Json(value).into_response()),
        Err(AdminError::Refused(refusal)) => {
            req.notices.error(refusal.message()).await;
            let dashboard = format!("{}/", admin.config.prefix.trim_end_matches('/'));
            Ok(Redirect::to(&dashboard).into_response())
        }
        Err(AdminError::Db(error)) => Err(Box::new(AppError::new(ErrorContext::database(error)))),
    }
}

/// A resource key that matches no registered model.
fn unknown_resource(resource: &str) -> AppResult<Response> {
    Err(Box::new(AppError::new(ErrorContext::not_found(resource))))
}

#[cfg(test)]
mod tests {
    use super::refonte_router;
    use crate::admin::refonte_admin::builder::site::AdminSite;
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use tower::ServiceExt;

    async fn status_of(path: &str) -> StatusCode {
        refonte_router("/admin", AdminSite::default())
            .oneshot(Request::builder().uri(path).body(Body::empty()).unwrap())
            .await
            .unwrap()
            .status()
    }

    /// Without the layers `build_admin_router` adds, a mounted route fails on
    /// its missing extensions: anything but a 404 proves the route exists.
    #[tokio::test]
    async fn the_list_and_detail_routes_are_mounted() {
        for path in ["/admin/next/notes/list", "/admin/next/notes/1/detail"] {
            assert_ne!(status_of(path).await, StatusCode::NOT_FOUND, "{path}");
        }
    }

    #[tokio::test]
    async fn a_path_outside_the_routes_is_not_found() {
        assert_eq!(status_of("/admin/next/notes").await, StatusCode::NOT_FOUND);
        assert_eq!(status_of("/admin/notes/list").await, StatusCode::NOT_FOUND);
    }
}
