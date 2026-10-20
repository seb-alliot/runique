//! Admin router: `build_admin_router` (login, dashboard, the layers around
//! every admin page) and the model pages, `{admin_path}/{resource}/list` and
//! `{admin_path}/{resource}/{id}/detail`. The model pages hold only their
//! routes: authentication, the current user, `noindex` and `AdminState` come
//! from the layers `build_admin_router` wraps around them, never copied.
pub mod admin_router;
pub mod detail;
pub mod list;

pub use admin_router::{ADMIN_TEMPLATE_SESSION_KEY, AdminState, build_admin_router};

use crate::admin::builder::registered::AdminError;
use crate::admin::builder::site::AdminSite;
use crate::admin::trad::{inject_admin_prefix, insert_admin_messages};
use crate::auth::session::CurrentUser;
use crate::context::template::{AppError, Request};
use crate::errors::error::ErrorContext;
use crate::utils::aliases::AppResult;
use axum::response::{IntoResponse, Redirect, Response};
use axum::{Extension, Router, routing::get};
use std::sync::Arc;

/// The rebuilt admin's routes under `admin_path`, sharing `site`.
pub(crate) fn refonte_router(admin_path: &str, site: Arc<AdminSite>) -> Router {
    Router::new()
        .route(
            &format!("{admin_path}/{{resource}}/list"),
            get(list::list_view),
        )
        .route(
            &format!("{admin_path}/{{resource}}/{{id}}/detail"),
            get(detail::detail_view),
        )
        .layer(Extension(site))
}

/// What the gate or the database said no with: a refusal as a flash message
/// and a redirect to the dashboard, a database failure as an error page.
async fn failure(error: AdminError, req: &Request, admin: &AdminState) -> AppResult<Response> {
    match error {
        AdminError::Refused(refusal) => {
            req.notices.error(refusal.message()).await;
            let dashboard = format!("{}/", admin.config.prefix.trim_end_matches('/'));
            Ok(Redirect::to(&dashboard).into_response())
        }
        AdminError::Db(error) => Err(Box::new(AppError::new(ErrorContext::database(error)))),
    }
}

/// The context every admin page shares: messages, prefix, site identity, the
/// signed-in user and the navigation, built here with the rights applied.
fn chrome(
    req: &mut Request,
    admin: &AdminState,
    site: &AdminSite,
    user: &CurrentUser,
    current_resource: &str,
) {
    insert_admin_messages(&mut req.context, "base");
    inject_admin_prefix(&mut req.context, &admin.config.prefix);
    let resources: Vec<serde_json::Value> = site
        .navigation(user, &admin.config.resource_order)
        .into_iter()
        .map(|entry| {
            serde_json::json!({
                "key": entry.key,
                "title": entry.title,
                "list_url": format!("{}/{}/list", admin.config.prefix, entry.key),
            })
        })
        .collect();
    req.context.insert("resources", &resources);
    req.context.insert("current_resource", current_resource);
    req.context.insert("current_user", user);
    req.context.insert("site_title", &admin.config.site_title);
    req.context
        .insert("site_url", &admin.config.view_site_href());
    req.context
        .insert("lang", crate::utils::trad::current_lang().code());
}

/// A resource key that matches no registered model.
fn unknown_resource(resource: &str) -> AppResult<Response> {
    Err(Box::new(AppError::new(ErrorContext::not_found(resource))))
}

#[cfg(test)]
mod tests {
    use super::refonte_router;
    use crate::admin::builder::site::AdminSite;
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use tower::ServiceExt;

    async fn status_of(path: &str) -> StatusCode {
        refonte_router("/admin", std::sync::Arc::new(AdminSite::default()))
            .oneshot(Request::builder().uri(path).body(Body::empty()).unwrap())
            .await
            .unwrap()
            .status()
    }

    /// Without the layers `build_admin_router` adds, a mounted route fails on
    /// its missing extensions: anything but a 404 proves the route exists.
    #[tokio::test]
    async fn the_list_and_detail_routes_are_mounted() {
        for path in ["/admin/notes/list", "/admin/notes/1/detail"] {
            assert_ne!(status_of(path).await, StatusCode::NOT_FOUND, "{path}");
        }
    }

    #[tokio::test]
    async fn a_path_outside_the_routes_is_not_found() {
        assert_eq!(status_of("/admin/notes").await, StatusCode::NOT_FOUND);
        assert_eq!(status_of("/admin/notes/1").await, StatusCode::NOT_FOUND);
        assert_eq!(status_of("/other/notes/list").await, StatusCode::NOT_FOUND);
    }
}
