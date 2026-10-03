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
use crate::auth::permissions::pull_groupes_db;
use crate::auth::session::{CurrentUser, get_user_id, logout};
use crate::auth::user_trait::RuniqueUser;
use crate::context::RequestExtensions;
use crate::utils::aliases::AEngine;
use crate::utils::config::TraceResult;

/// Middleware: admin access required (is_staff OR is_superuser)
///
/// - No matched route → 404 (passes through, Axum handles it)
/// - Unauthenticated → redirect to `{prefix}/login`
pub(crate) async fn admin_required(
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

    let account = crate::auth::BuiltinUserEntity::find_by_id(&engine.db, user_id)
        .await
        .filter(|user| user.can_access_admin());
    match account {
        Some(account) => {
            let groupes = pull_groupes_db(&engine.db, user_id).await;
            let current_user = CurrentUser {
                id: account.id,
                username: account.username,
                is_staff: account.is_staff,
                is_superuser: account.is_superuser,
                groupes,
            };
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
