//! Admin router: `build_admin_router` (login, dashboard, the layers around
//! every admin page) and the model pages, `{prefix}/{resource}/list` and
//! `{prefix}/{resource}/{id}/detail`. The model pages hold only their
//! routes: authentication, the current user, `noindex` and `AdminState` come
//! from the layers `build_admin_router` wraps around them, never copied.
pub mod admin_router;
pub mod bulk;
pub mod context;
pub mod create;
pub mod delete;
pub mod detail;
pub mod edit;
pub mod history;
pub mod list;
pub mod rank;
pub mod reset_password;

pub use admin_router::{ADMIN_TEMPLATE_SESSION_KEY, AdminState, build_admin_router};

/// Whether a declarative admin route replaces a generated view or stands alone.
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub enum AdminView {
    Detail,
    /// A standalone page inside the admin boundary, not an override.
    Standalone,
}

use crate::admin::builder::registered::AdminError;
use crate::admin::builder::site::AdminSite;
use crate::admin::trad::{inject_admin_prefix, insert_admin_messages};
use crate::auth::current_user::CurrentUser;
use crate::context::template::{AppError, Request};
use crate::errors::error::ErrorContext;
use crate::macros::routeur::register_url::reverse_with_parameters;
use crate::urlpatterns;
use crate::utils::aliases::AppResult;
use crate::utils::constante::admin_key::admin_route;
use axum::response::{IntoResponse, Redirect, Response};
use axum::{Extension, Router, routing::get};
use std::sync::Arc;

/// The rebuilt admin's routes under `prefix`, the admin's public base (mount
/// prefix included), sharing `site`. Named in `urlpatterns!` like the fixed
/// routes, so `{% link "admin_list" resource=… %}` resolves the path served.
pub(crate) fn refonte_router(prefix: &str, site: Arc<AdminSite>) -> Router {
    urlpatterns! {
        &format!("{prefix}/{{resource}}/list") => get(list::list_view), name = admin_route::LIST,
        &format!("{prefix}/{{resource}}/{{id}}/detail") => get(detail::detail_view), name = admin_route::DETAIL,
        &format!("{prefix}/{{resource}}/create") => get(create::create_view).post(create::create_view), name = admin_route::CREATE,
        &format!("{prefix}/{{resource}}/rank") => axum::routing::post(rank::rank_view), name = admin_route::RANK,
        &format!("{prefix}/{{resource}}/bulk") => axum::routing::post(bulk::bulk_view), name = admin_route::BULK,
        &format!("{prefix}/{{resource}}/{{id}}/reset-password") => axum::routing::post(reset_password::reset_password_view), name = admin_route::RESET_PASSWORD,
        &format!("{prefix}/history") => get(history::history_view), name = admin_route::HISTORY,
        &format!("{prefix}/history/{{id}}") => get(history::history_entry_view), name = admin_route::HISTORY_ENTRY,
        &format!("{prefix}/{{resource}}/{{id}}/delete") => get(delete::delete_view).post(delete::delete_view), name = admin_route::DELETE,
        &format!("{prefix}/{{resource}}/{{id}}/edit") => get(edit::edit_view).post(edit::edit_view), name = admin_route::EDIT,
    }
    .into_router()
    .layer(Extension(site))
}

/// The path of the named admin route `name`, built like `{% link %}` builds it
/// in a template. Empty, and logged, if the name or a parameter is wrong: a
/// broken link, never a broken page.
fn link(req: &Request, name: &str, parameters: &[(&str, &str)]) -> String {
    reverse_with_parameters(&req.engine, name, parameters).unwrap_or_else(|error| {
        tracing::error!(route = name, error = ?error, "admin link not resolved");
        String::new()
    })
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
                "list_url": link(req, admin_route::LIST, &[("resource", entry.key)]),
            })
        })
        .collect();
    req.context.insert("resources", &resources);
    req.context.insert("current_resource", current_resource);
    req.context.insert("current_user", user);
    // The history entry only when there is a history this account may read.
    if !site.history_scopes(&req.engine.db, user).is_empty() {
        let history_url = link(req, admin_route::HISTORY, &[]);
        req.context.insert("history_url", &history_url);
    }
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
