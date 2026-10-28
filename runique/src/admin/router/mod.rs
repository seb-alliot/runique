//! Admin router: `build_admin_router` (login, dashboard, the layers around
//! every admin page) and the model pages, `{prefix}/{resource}/list` and
//! `{prefix}/{resource}/{id}/detail`. The model pages hold only their
//! routes: authentication, the current user, `noindex` and `AdminState` come
//! from the layers `build_admin_router` wraps around them, never copied.
pub mod account;
pub mod admin_router;
pub mod context;
pub mod crud;
pub mod history;
pub mod path;
pub mod permission;

pub use admin_router::{ADMIN_TEMPLATE_SESSION_KEY, AdminState, build_admin_router};
pub use path::AdminPaths;

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
use crate::urlpatterns;
use crate::utils::aliases::AppResult;
use crate::utils::constante::admin_key::admin_route;
use axum::response::{IntoResponse, Redirect, Response};
use axum::{Extension, Router, routing::get};
use context::NavLinks;
use std::sync::Arc;

/// The model pages, served at `paths`, sharing `site`. Still named in
/// `urlpatterns!`, for a project's own templates (`{% link "admin_list" … %}`).
pub(crate) fn refonte_router(paths: &AdminPaths, site: Arc<AdminSite>) -> Router {
    urlpatterns! {
        &paths.list => get(crud::list::list_view), name = admin_route::LIST,
        &paths.detail => get(crud::detail::detail_view), name = admin_route::DETAIL,
        &paths.create => get(crud::create::create_view).post(crud::create::create_view), name = admin_route::CREATE,
        &paths.rights => get(permission::rights::rights_view).post(permission::rights::rights_view), name = admin_route::RIGHTS,
        &paths.rank => axum::routing::post(permission::rank::rank_view), name = admin_route::RANK,
        &paths.bulk => axum::routing::post(crud::bulk::bulk_view), name = admin_route::BULK,
        &paths.reset_password => axum::routing::post(account::reset_password::reset_password_view), name = admin_route::RESET_PASSWORD,
        &paths.history => get(history::history_view), name = admin_route::HISTORY,
        &paths.history_entry => get(history::history_entry_view), name = admin_route::HISTORY_ENTRY,
        &paths.delete => get(crud::delete::delete_view).post(crud::delete::delete_view), name = admin_route::DELETE,
        &paths.edit => get(crud::edit::edit_view).post(crud::edit::edit_view), name = admin_route::EDIT,
    }
    .into_router()
    .layer(Extension(site))
}

/// What the gate or the database said no with: a refusal as a flash message
/// and a redirect to the dashboard, a database failure as an error page.
async fn failure(error: AdminError, req: &Request, admin: &AdminState) -> AppResult<Response> {
    match error {
        AdminError::Refused(refusal) => {
            req.notices.error(refusal.message()).await;
            Ok(Redirect::to(&admin.paths.dashboard).into_response())
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
                "list_url": admin.paths.list_of(entry.key),
            })
        })
        .collect();
    req.context.insert("resources", &resources);
    req.context.insert("current_resource", current_resource);
    req.context.insert("current_user", user);
    let paths = &admin.paths;
    let nav = NavLinks {
        dashboard: paths.dashboard.clone(),
        // Only when there is a history this account may read.
        history: (!site.history_scopes(&req.engine.db, user).is_empty())
            .then(|| paths.history.clone()),
        logout: paths.logout.clone(),
        site: admin.config.view_site_href().to_string(),
        toggle_template: paths.toggle_template.clone(),
    };
    req.context.insert("nav", &nav);
    req.context.insert("site_title", &admin.config.site_title);
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
        refonte_router(
            &super::AdminPaths::new("/admin"),
            std::sync::Arc::new(AdminSite::default()),
        )
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
