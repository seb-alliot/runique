//! Axum middlewares for the admin space — redirection if not authenticated, permission verification.
use std::sync::Arc;

use axum::{
    Extension,
    extract::{MatchedPath, Request},
    http::{HeaderMap, header},
    middleware::Next,
    response::{IntoResponse, Redirect, Response},
};
use tower_sessions::Session;

use crate::admin::router::admin_router::AdminState;
use crate::auth::user_trait::RuniqueUser;
use crate::auth::{
    current_user::CurrentUser,
    permissions::load_rights,
    permissions::rang::Rang,
    session::{get_user_id, logout},
};
use crate::context::RequestExtensions;
use crate::utils::aliases::AEngine;
use crate::utils::config::TraceResult;
use crate::utils::pk::Pk;
use crate::utils::trad::t;

/// Middleware: admin access required (a rank, or a staff group)
///
/// - No matched route → 404 (passes through, Axum handles it)
/// - Unauthenticated → redirect to `{prefix}/login`
pub(crate) async fn admin_access(
    Extension(admin): Extension<Arc<AdminState>>,
    request: Request,
    next: Next,
) -> Response {
    if request.extensions().get::<MatchedPath>().is_none() {
        return next.run(request).await;
    }
    // `load_admin_user` (outer layer) put the account here only if the
    // database still lets it into the admin.
    let authed = request
        .extensions()
        .get::<CurrentUser>()
        .is_some_and(CurrentUser::can_access_admin);
    // Trace the admin access gate (rank or staff group), so the
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
            .get::<crate::auth::current_user::CurrentUser>();
        let path = request
            .extensions()
            .get::<MatchedPath>()
            .map_or("", MatchedPath::as_str);
        crate::runique_log!(
            level,
            path = %path,
            user = cu.map_or("-", |u| u.username.as_str()),
            rang = ?cu.map(|u| u.rang),
            is_staff = cu.is_some_and(CurrentUser::is_staff),
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

/// Middleware: the signed-in admin as the database knows them now.
///
/// The session only says who signed in. The account's state (active, staff,
/// superuser) and its groups' rights are read here on every admin request, so a
/// change made anywhere — the admin, the CLI, raw SQL, another instance —
/// applies to the very next request. An account that can no longer use the
/// admin has its session closed.
pub(crate) async fn load_admin_user(
    session: Session,
    mut request: Request,
    next: Next,
) -> Response {
    // The app-wide auth middleware already put a `CurrentUser` built from the
    // session — the very copy this middleware exists to replace.
    request.extensions_mut().remove::<CurrentUser>();
    let Some(user_id) = get_user_id(&session).await else {
        if session.id().is_some() {
            session.delete().await.trace(
                crate::utils::runique_log::get_log()
                    .session
                    .as_ref()
                    .and_then(|s| s.store),
                "delete anonymous session",
            );
        }
        return next.run(request).await;
    };
    let Some(engine) = request.extensions().get::<AEngine>().cloned() else {
        return next.run(request).await;
    };

    // Staff access comes from the groups: the account alone can't decide, so
    // the user is built first and judged after.
    let current_user = fresh_user(&engine, user_id)
        .await
        .filter(CurrentUser::can_access_admin);
    match current_user {
        Some(current_user) => {
            RequestExtensions::new()
                .with_current_user(current_user)
                .inject_request(&mut request);
        }
        None => {
            if let Some(level) = crate::utils::runique_log::get_log()
                .admin
                .as_ref()
                .and_then(|a| a.auth)
            {
                crate::runique_log!(level, user_id = %user_id, "admin account gone, inactive or no longer staff — session closed");
            }
            let db_store = engine
                .session_db_store
                .read()
                .ok()
                .and_then(|g| g.as_ref().cloned());
            logout(&session, db_store.as_deref()).await.trace(
                crate::utils::runique_log::get_log()
                    .session
                    .as_ref()
                    .and_then(|s| s.store),
                "close the session of an account that lost admin access",
            );
        }
    }
    next.run(request).await
}

/// The account behind `user_id` as the database knows it now, with its
/// rights; `None` once it is gone or may no longer sign in.
async fn fresh_user(engine: &AEngine, user_id: Pk) -> Option<CurrentUser> {
    let account = crate::auth::BuiltinUserEntity::find_by_id(&engine.db, user_id)
        .await
        .filter(RuniqueUser::can_sign_in)?;
    let mut user = CurrentUser {
        id: account.id,
        rang: Rang::from(&account),
        username: account.username,
        groupes: Vec::new(),
        droits: Vec::new(),
    };
    load_rights(&engine.db, &mut user).await;
    Some(user)
}

/// Middleware of the routes declared `admin` in `urlpatterns!`: the account and
/// rights read fresh, as on the admin's pages. These routes live at the site's
/// own paths, so nothing here may show the admin's prefix to a stranger or end
/// a site session.
///
/// - Signed in with admin access → the handler.
/// - Signed in without it → `/` with a notice, the session left open.
/// - Anonymous → `/`, or the admin login when the previous page was the
///   admin's own.
pub(crate) async fn declared_route_access(
    Extension(admin): Extension<Arc<AdminState>>,
    session: Session,
    mut request: Request,
    next: Next,
) -> Response {
    request.extensions_mut().remove::<CurrentUser>();
    let engine = request.extensions().get::<AEngine>().cloned();
    let user = match (get_user_id(&session).await, engine) {
        (Some(user_id), Some(engine)) => fresh_user(&engine, user_id).await,
        _ => None,
    };
    match user {
        Some(user) if user.can_access_admin() => {
            RequestExtensions::new()
                .with_current_user(user)
                .inject_request(&mut request);
            next.run(request).await
        }
        Some(user) => {
            if let Some(level) = crate::utils::runique_log::get_log()
                .admin
                .as_ref()
                .and_then(|a| a.auth)
            {
                crate::runique_log!(level, user = %user.username, "declared admin route refused: no admin access");
            }
            let notices = crate::flash::flash_manager::Message { session };
            notices.error(t("admin.access.action_not_allowed")).await;
            Redirect::to("/").into_response()
        }
        None => {
            let prefix = admin.config.prefix.trim_end_matches('/');
            if came_from_admin(request.headers(), prefix) {
                Redirect::to(&format!("{prefix}/login")).into_response()
            } else {
                Redirect::to("/").into_response()
            }
        }
    }
}

/// Whether the `Referer` is a page under the admin's `prefix`, compared by
/// segment (`/admin` is not `/administrator`). Safe to trust: only someone who
/// already knows the prefix can send one that matches.
fn came_from_admin(headers: &HeaderMap, prefix: &str) -> bool {
    let Some(referer) = headers.get(header::REFERER).and_then(|v| v.to_str().ok()) else {
        return false;
    };
    // `https://host/path?query` → `/path`
    let path = referer
        .split_once("://")
        .and_then(|(_, rest)| rest.find('/').map(|i| &rest[i..]))
        .unwrap_or_default()
        .split(['?', '#'])
        .next()
        .unwrap_or_default();
    !prefix.is_empty()
        && path
            .strip_prefix(prefix)
            .is_some_and(|rest| rest.is_empty() || rest.starts_with('/'))
}

#[cfg(test)]
mod came_from_admin_tests {
    use super::came_from_admin;
    use axum::http::{HeaderMap, HeaderValue, header};

    fn headers(referer: &'static str) -> HeaderMap {
        let mut h = HeaderMap::new();
        h.insert(header::REFERER, HeaderValue::from_static(referer));
        h
    }

    #[test]
    fn an_admin_page_is_recognised() {
        for referer in [
            "https://site.test/secret/admin",
            "https://site.test/secret/admin/",
            "https://site.test/secret/admin/users/list?page=2",
            "http://site.test:8000/secret/admin#top",
        ] {
            assert!(
                came_from_admin(&headers(referer), "/secret/admin"),
                "{referer}"
            );
        }
    }

    #[test]
    fn anything_else_is_not() {
        assert!(!came_from_admin(&HeaderMap::new(), "/secret/admin"));
        for referer in [
            "https://site.test/secret/administrator",
            "https://site.test/secret",
            "https://site.test/",
            "https://site.test",
            "https://site.test/x/secret/admin",
            "https://site.test/?next=/secret/admin",
            "/secret/admin",
        ] {
            assert!(
                !came_from_admin(&headers(referer), "/secret/admin"),
                "{referer}"
            );
        }
    }

    #[test]
    fn an_empty_prefix_never_matches() {
        assert!(!came_from_admin(&headers("https://site.test/"), ""));
    }
}
