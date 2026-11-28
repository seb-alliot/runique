//! Admin router: `build_admin_router` (login, dashboard, the layers around
//! every admin page) and the model pages, `{prefix}/{resource}/list` and
//! `{prefix}/{resource}/{id}/detail`. The model pages hold only their
//! routes: authentication, the current user, `noindex` and `AdminState` come
//! from the layers `build_admin_router` wraps around them, never copied.
use crate::{admin::template::Page, translation::current_lang};
use serde_json::Value;
pub mod account;
pub mod context;
pub mod crud;
pub mod history;
pub mod path;
pub mod permission;
pub mod routes;

pub use path::AdminPaths;
pub use routes::{ADMIN_TEMPLATE_SESSION_KEY, AdminState, build_admin_router};

/// Whether a declarative admin route replaces a generated view or stands alone.
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub enum AdminView {
    Detail,
    /// A standalone page inside the admin boundary, not an override.
    Standalone,
}

use crate::{
    admin::{
        builder::{registered::AdminError, site::AdminSite},
        constants::route_names,
        translation::{inject_admin_prefix, insert_admin_messages},
    },
    auth::current_user::CurrentUser,
    errors::{AppError, ErrorContext},
    http::Request,
    urlpatterns,
    utils::aliases::AppResult,
};
use axum::{
    Extension, Router,
    response::{IntoResponse, Redirect, Response},
    routing::get,
};
use context::NavLinks;
use std::sync::Arc;

/// The model pages, served at `paths`, sharing `site`. Still named in
/// `urlpatterns!`, for a project's own templates (`{% link "admin_list" … %}`).
pub(crate) fn refonte_router(paths: &AdminPaths, site: Arc<AdminSite>) -> Router {
    urlpatterns! {
        &paths.list => get(crud::list::list_view), name = route_names::LIST,
        &paths.detail => get(crud::detail::detail_view), name = route_names::DETAIL,
        &paths.create => get(crud::create::create_view).post(crud::create::create_view), name = route_names::CREATE,
        &paths.rights => get(permission::rights::rights_view).post(permission::rights::rights_view), name = route_names::RIGHTS,
        &paths.rank => axum::routing::post(permission::rank::rank_view), name = route_names::RANK,
        &paths.group_select => get(permission::rights::group_select_view), name = route_names::GROUP_SELECT,
        &paths.groups => axum::routing::post(account::memberships::memberships_view), name = route_names::GROUPS,
        &paths.transfer => get(permission::transfer::transfer_view).post(permission::transfer::transfer_view), name = route_names::TRANSFER,
        &paths.bulk => axum::routing::post(crud::bulk::bulk_view), name = route_names::BULK,
        &paths.reset_password => axum::routing::post(account::reset_password::reset_password_view), name = route_names::RESET_PASSWORD,
        &paths.history => get(history::history_view), name = route_names::HISTORY,
        &paths.history_entry => get(history::history_entry_view), name = route_names::HISTORY_ENTRY,
        &paths.delete => get(crud::delete::delete_view).post(crud::delete::delete_view), name = route_names::DELETE,
        &paths.edit => get(crud::edit::edit_view).post(crud::edit::edit_view), name = route_names::EDIT,
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
    page: Page,
) {
    insert_admin_messages(&mut req.context, "base");
    req.context
        .insert("stylesheets", admin.sheets.of(page, current_resource));
    inject_admin_prefix(&mut req.context, &admin.config.prefix);
    let resources: Vec<Value> = site
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
    req.context.insert("lang", current_lang().code());
}

/// What a page declared `admin` in `urlpatterns!` needs to extend
/// `admin_base.html`: stylesheets, menu with the rights applied, top bar and site
/// identity. `page` picks the stylesheets to load, `current_resource` the model
/// the menu highlights. It only prepares the display: what the handler reads or
/// writes still goes through the gate. Without a signed-in user, it adds nothing.
pub fn admin_page_context(
    req: &mut Request,
    admin: &AdminState,
    site: &AdminSite,
    page: Page,
    current_resource: &str,
) {
    let Some(user) = req.user.clone() else {
        return;
    };
    chrome(req, admin, site, &user, current_resource, page);
}

/// A resource key that matches no registered model.
fn unknown_resource(resource: &str) -> AppResult<Response> {
    Err(Box::new(AppError::new(ErrorContext::not_found(resource))))
}

#[cfg(test)]
mod tests {
    use super::refonte_router;
    use crate::admin::builder::site::AdminSite;
    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
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
