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
    let admin_user = request
        .extensions()
        .get::<CurrentUser>()
        .filter(|u| u.can_access_admin())
        .cloned();
    let authed = admin_user.is_some();
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
    let Some(user) = admin_user else {
        let login_url = format!("{}/login", admin.config.prefix.trim_end_matches('/'));
        return Redirect::to(&login_url).into_response();
    };
    let mut request = request;
    request
        .extensions_mut()
        .insert(crate::auth::gate::AdminConstraint(user));
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
            request
                .extensions_mut()
                .insert(crate::auth::gate::AdminConstraint(user.clone()));
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

#[cfg(all(test, feature = "sqlite"))]
mod admin_access_constraint_tests {
    use super::admin_access;
    use crate::admin::router::admin_router::AdminState;
    use crate::auth::{
        current_user::CurrentUser,
        gate::{AdminConstraint, test_support::fixtures::pk},
        permissions::{Groupe, rang::Rang},
    };
    use axum::{
        Extension, Router,
        body::Body,
        extract::Request,
        http::{StatusCode, header},
        middleware::{self, Next},
        routing::get,
    };
    use std::sync::Arc;
    use tower::ServiceExt;

    fn user(rang: Rang, staff_group: bool) -> CurrentUser {
        CurrentUser {
            id: pk(1),
            username: "tester".into(),
            rang,
            groupes: if staff_group {
                vec![Groupe {
                    id: 1,
                    nom: "relecteurs".into(),
                    is_staff: true,
                    permissions: Vec::new(),
                }]
            } else {
                Vec::new()
            },
            droits: Vec::new(),
        }
    }

    async fn state(prefix: &str) -> Extension<Arc<AdminState>> {
        let mut config = crate::admin::config::AdminConfig::new();
        config.prefix = prefix.to_string();
        Extension(Arc::new(AdminState {
            config: Arc::new(config),
            login_guard: None,
        }))
    }

    /// What a handler behind `admin_access` sees of the constraint.
    async fn seen(req: Request) -> &'static str {
        if req.extensions().get::<AdminConstraint>().is_some() {
            "constrained"
        } else {
            "free"
        }
    }

    /// `admin_access` as the admin's router mounts it: after a layer that
    /// stands for `load_admin_user` and puts the account (or nothing).
    async fn call(
        router: Router,
        account: Option<CurrentUser>,
        prefix: &str,
        path: &str,
    ) -> (StatusCode, Option<String>, String) {
        let app = router
            .layer(middleware::from_fn(admin_access))
            .layer(middleware::from_fn(move |mut req: Request, next: Next| {
                let account = account.clone();
                async move {
                    if let Some(account) = account {
                        req.extensions_mut().insert(account);
                    }
                    next.run(req).await
                }
            }))
            .layer(state(prefix).await);
        let resp = app
            .oneshot(Request::builder().uri(path).body(Body::empty()).unwrap())
            .await
            .unwrap();
        let status = resp.status();
        let location = resp
            .headers()
            .get(header::LOCATION)
            .and_then(|v| v.to_str().ok())
            .map(str::to_owned);
        let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
            .await
            .unwrap();
        (status, location, String::from_utf8(body.to_vec()).unwrap())
    }

    /// The account the constraint carries, next to the one of the extension.
    async fn who(req: Request) -> String {
        let carried = req.extensions().get::<AdminConstraint>().map(|c| &c.0);
        let ext = req.extensions().get::<CurrentUser>();
        match (carried, ext) {
            (Some(c), Some(e)) if c.id == e.id && c.username == e.username => {
                format!("{}:{}", c.id, c.username)
            }
            _ => "mismatch".to_string(),
        }
    }

    fn named(name: &str, id: u8, rang: Rang) -> CurrentUser {
        CurrentUser {
            id: pk(id),
            username: name.into(),
            ..user(rang, false)
        }
    }

    #[tokio::test]
    async fn the_constraint_carries_the_admitted_account() {
        let router = || Router::new().route("/who", get(who));
        for (label, account) in [
            ("admin", named("alice", 2, Rang::Admin)),
            ("dirigeant", named("bob", 3, Rang::Dirigeant)),
            ("superuser", named("carol", 4, Rang::Superuser)),
            (
                "staff group member",
                CurrentUser {
                    id: pk(5),
                    username: "dave".into(),
                    ..user(Rang::Lambda, true)
                },
            ),
        ] {
            let expected = format!("{}:{}", account.id, account.username);
            let (status, _, body) = call(router(), Some(account), "/admin", "/who").await;
            assert_eq!(status, StatusCode::OK, "{label}");
            assert_eq!(body, expected, "{label}");
        }
    }

    #[tokio::test]
    async fn the_constraint_never_carries_a_refused_account() {
        let router = Router::new().route("/who", get(who));
        let (status, _, body) = call(
            router,
            Some(named("eve", 6, Rang::Lambda)),
            "/admin",
            "/who",
        )
        .await;
        assert_eq!(status, StatusCode::SEE_OTHER);
        assert!(
            !body.contains("eve") && !body.contains("mismatch"),
            "{body}"
        );
    }

    fn panel() -> Router {
        Router::new().route("/panel", get(seen))
    }

    #[tokio::test]
    async fn every_account_with_admin_access_reaches_the_handler_constrained() {
        for (label, account) in [
            ("admin", user(Rang::Admin, false)),
            ("dirigeant", user(Rang::Dirigeant, false)),
            ("superuser", user(Rang::Superuser, false)),
            ("staff group member", user(Rang::Lambda, true)),
        ] {
            let (status, location, body) = call(panel(), Some(account), "/admin", "/panel").await;
            assert_eq!(status, StatusCode::OK, "{label}");
            assert_eq!(location, None, "{label}");
            assert_eq!(body, "constrained", "{label}");
        }
    }

    #[tokio::test]
    async fn without_admin_access_the_login_is_shown_and_the_handler_not_reached() {
        for (label, account) in [
            ("anonymous", None),
            ("lambda without group", Some(user(Rang::Lambda, false))),
        ] {
            let (status, location, body) = call(panel(), account, "/admin", "/panel").await;
            assert_eq!(status, StatusCode::SEE_OTHER, "{label}");
            assert_eq!(location.as_deref(), Some("/admin/login"), "{label}");
            assert!(
                !body.contains("constrained") && !body.contains("free"),
                "{label}"
            );
        }
    }

    #[tokio::test]
    async fn the_login_redirect_follows_the_mount_prefix_without_double_slash() {
        let (_, location, _) = call(panel(), None, "/mnt/secret-admin/", "/panel").await;
        assert_eq!(location.as_deref(), Some("/mnt/secret-admin/login"));
    }

    /// No matched route: the request goes on untouched, the constraint is
    /// not given to something the panel doesn't serve.
    #[tokio::test]
    async fn an_unmatched_path_gets_no_constraint() {
        let router = Router::new().fallback(seen);
        let (status, _, body) =
            call(router, Some(user(Rang::Admin, false)), "/admin", "/nowhere").await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(body, "free");
    }
}

#[cfg(all(test, feature = "sqlite"))]
mod declared_route_constraint_tests {
    use super::declared_route_access;
    use crate::admin::router::admin_router::AdminState;
    use crate::auth::{
        current_user::CurrentUser,
        gate::{AdminConstraint, test_support::fixtures::pk},
        user,
    };
    use crate::config::RuniqueConfig;
    use crate::engine::RuniqueEngine;
    use crate::utils::{
        aliases::AEngine, constante::session_key::session::SESSION_USER_ID_KEY, pk::Pk,
    };
    use axum::{
        Extension, Router,
        body::Body,
        extract::Request,
        http::StatusCode,
        middleware::{self, Next},
        routing::get,
    };
    use sea_orm::{ActiveModelTrait, ActiveValue::Set, ConnectionTrait, Schema};
    use std::sync::Arc;
    use tower::ServiceExt;
    use tower_sessions::{MemoryStore, Session};

    /// Two admins, to tell the right one, and a plain member.
    async fn engine() -> AEngine {
        let conn = sea_orm::Database::connect("sqlite::memory:").await.unwrap();
        conn.execute(
            &Schema::new(sea_orm::DatabaseBackend::Sqlite).create_table_from_entity(user::Entity),
        )
        .await
        .unwrap();
        let now = chrono::NaiveDate::from_ymd_opt(2026, 1, 1)
            .unwrap()
            .and_hms_opt(0, 0, 0)
            .unwrap();
        for (id, name, active, admin) in [
            (pk(1), "alice", true, true),
            (pk(2), "bob", true, true),
            (pk(3), "member", true, false),
        ] {
            user::ActiveModel {
                id: Set(id),
                username: Set(name.into()),
                email: Set(format!("{name}@example.com")),
                password: Set("x".into()),
                is_active: Set(active),
                is_dirigeant: Set(false),
                is_admin: Set(admin),
                is_superuser: Set(false),
                activated_at: Set(Some(now)),
                ..Default::default()
            }
            .insert(&conn)
            .await
            .unwrap();
        }
        Arc::new(RuniqueEngine::new(
            RuniqueConfig::default(),
            tera::Tera::default(),
            conn,
        ))
    }

    /// The constraint's account and the extension's, from the handler.
    async fn who(req: Request) -> String {
        let carried = req.extensions().get::<AdminConstraint>().map(|c| &c.0);
        let ext = req.extensions().get::<CurrentUser>();
        match (carried, ext) {
            (Some(c), Some(e)) if c.id == e.id && c.username == e.username => {
                format!("{}:{}", c.id, c.username)
            }
            (None, None) => "none".to_string(),
            _ => "mismatch".to_string(),
        }
    }

    async fn call(engine: AEngine, signed_in: Option<Pk>) -> (StatusCode, String) {
        let session = Session::new(None, Arc::new(MemoryStore::default()), None);
        if let Some(id) = signed_in {
            session.insert(SESSION_USER_ID_KEY, id).await.unwrap();
        }
        let state = Extension(Arc::new(AdminState {
            config: Arc::new(crate::admin::config::AdminConfig::new()),
            login_guard: None,
        }));
        let app = Router::new()
            .route("/page", get(who))
            .layer(middleware::from_fn(declared_route_access))
            .layer(state)
            .layer(middleware::from_fn(move |mut req: Request, next: Next| {
                let engine = engine.clone();
                let session = session.clone();
                async move {
                    req.extensions_mut().insert(engine);
                    req.extensions_mut().insert(session);
                    next.run(req).await
                }
            }));
        let resp = app
            .oneshot(Request::builder().uri("/page").body(Body::empty()).unwrap())
            .await
            .unwrap();
        let status = resp.status();
        let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
            .await
            .unwrap();
        (status, String::from_utf8(body.to_vec()).unwrap())
    }

    #[tokio::test]
    async fn the_constraint_carries_the_account_read_from_the_session() {
        let engine = engine().await;
        let (status, body) = call(engine.clone(), Some(pk(1))).await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(body, format!("{}:alice", pk(1)));
        let (status, body) = call(engine, Some(pk(2))).await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(body, format!("{}:bob", pk(2)));
    }

    #[tokio::test]
    async fn a_refused_account_gets_no_constraint_and_no_handler() {
        let engine = engine().await;
        for (label, signed_in) in [
            ("member", Some(pk(3))),
            ("unknown id", Some(pk(9))),
            ("anonymous", None),
        ] {
            let (status, body) = call(engine.clone(), signed_in).await;
            assert_eq!(status, StatusCode::SEE_OTHER, "{label}");
            assert_eq!(body, "", "{label}");
        }
    }
}
