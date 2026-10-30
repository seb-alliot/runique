//! The CSRF token changes whenever the privilege level does: at login, like
//! the session id, and at logout.
//!
//! The attack this closes: someone plants an anonymous session on a victim
//! (session fixation) and so knows its CSRF token. `login()` already cycles the
//! session id, but the token used to survive the login, so the attacker could
//! still forge requests the victim's logged-in session would accept.

use crate::helpers::{assert::assert_status, pk::pk, server::build_engine, user::test_user};
#[cfg(feature = "orm")]
use axum::extract::State;
use axum::{
    Router,
    body::Body,
    http::{Request, Response},
    middleware,
    routing::{get, post},
};
#[cfg(feature = "orm")]
use runique::middleware::session::RuniqueSessionStore;
use runique::{
    auth::session::{login, logout},
    middleware::{security::csrf::csrf_middleware, session::CleaningMemoryStore},
    utils::crypto::csrf::CsrfToken,
};
#[cfg(feature = "orm")]
use std::sync::Arc;
use tower::ServiceExt;
use tower_sessions::{MemoryStore, Session, SessionManagerLayer};

async fn log_in(session: Session) -> &'static str {
    login(&session, &test_user(pk(1), "setsuna", false), None, false)
        .await
        .expect("login");
    "logged in"
}

async fn log_out(session: Session) -> &'static str {
    logout(&session, None).await.expect("logout");
    "logged out"
}

async fn app() -> Router {
    Router::new()
        .route("/", get(|| async { "ok" }))
        .route("/login", post(log_in))
        .route("/logout", post(log_out))
        .route("/submit", post(|| async { "submitted" }))
        .layer(middleware::from_fn_with_state(
            build_engine().await,
            csrf_middleware,
        ))
        .layer(SessionManagerLayer::new(MemoryStore::default()))
}

fn post_with(uri: &str, cookie: &str, masked_token: &str) -> Request<Body> {
    Request::builder()
        .method("POST")
        .uri(uri)
        .header("cookie", cookie)
        .header("X-CSRF-Token", masked_token)
        .body(Body::empty())
        .expect("request")
}

/// The `name=value` part of the session cookie the response sets.
fn session_cookie(resp: &Response<Body>) -> String {
    let header = resp
        .headers()
        .get("set-cookie")
        .expect("the response should set the session cookie")
        .to_str()
        .expect("ASCII cookie");
    header.split(';').next().unwrap_or(header).to_string()
}

fn masked_token(resp: &Response<Body>) -> String {
    resp.headers()
        .get("X-CSRF-Token")
        .expect("the response should carry a CSRF token")
        .to_str()
        .expect("ASCII token")
        .to_string()
}

fn raw(masked: &str) -> String {
    CsrfToken::unmasked(masked)
        .expect("a well-formed masked token")
        .as_str()
        .to_string()
}

#[tokio::test]
async fn login_hands_out_a_new_csrf_token_and_the_old_one_stops_working() {
    let app = app().await;

    // An anonymous visit: this is the session and token an attacker would plant.
    let visit = app
        .clone()
        .oneshot(Request::get("/").body(Body::empty()).expect("request"))
        .await
        .expect("GET /");
    let anonymous_cookie = session_cookie(&visit);
    let anonymous_token = masked_token(&visit);

    // The victim logs in on that session.
    let logged_in = app
        .clone()
        .oneshot(post_with("/login", &anonymous_cookie, &anonymous_token))
        .await
        .expect("POST /login");
    assert_status(&logged_in, 200);
    let user_cookie = session_cookie(&logged_in);
    let user_token = masked_token(&logged_in);
    assert_ne!(
        raw(&user_token),
        raw(&anonymous_token),
        "the CSRF token must change at login"
    );

    // The attacker's token is now refused on the victim's session...
    let forged = app
        .clone()
        .oneshot(post_with("/submit", &user_cookie, &anonymous_token))
        .await
        .expect("POST /submit with the old token");
    assert_status(&forged, 403);

    // ...while the token the login response handed out works.
    let genuine = app
        .clone()
        .oneshot(post_with("/submit", &user_cookie, &user_token))
        .await
        .expect("POST /submit with the new token");
    assert_status(&genuine, 200);
}

#[tokio::test]
async fn logout_hands_out_a_new_csrf_token_and_the_old_one_stops_working() {
    let app = app().await;

    let visit = app
        .clone()
        .oneshot(Request::get("/").body(Body::empty()).expect("request"))
        .await
        .expect("GET /");
    let logged_in = app
        .clone()
        .oneshot(post_with(
            "/login",
            &session_cookie(&visit),
            &masked_token(&visit),
        ))
        .await
        .expect("POST /login");
    let user_cookie = session_cookie(&logged_in);
    let user_token = masked_token(&logged_in);

    let logged_out = app
        .clone()
        .oneshot(post_with("/logout", &user_cookie, &user_token))
        .await
        .expect("POST /logout");
    assert_status(&logged_out, 200);
    let anonymous_cookie = session_cookie(&logged_out);
    let anonymous_token = masked_token(&logged_out);
    assert_ne!(
        raw(&anonymous_token),
        raw(&user_token),
        "the CSRF token must change at logout"
    );

    let stale = app
        .clone()
        .oneshot(post_with("/submit", &anonymous_cookie, &user_token))
        .await
        .expect("POST /submit with the logged-in token");
    assert_status(&stale, 403);

    let fresh = app
        .clone()
        .oneshot(post_with("/submit", &anonymous_cookie, &anonymous_token))
        .await
        .expect("POST /submit with the new token");
    assert_status(&fresh, 200);
}

/// The password reset page calls `logout()` on every visit, mostly from an
/// anonymous session: that must leave its CSRF token alone, or a form shown
/// again on the same page would carry a dead one.
#[tokio::test]
async fn logout_without_anyone_logged_in_keeps_the_csrf_token() {
    let app = app().await;

    let visit = app
        .clone()
        .oneshot(Request::get("/").body(Body::empty()).expect("request"))
        .await
        .expect("GET /");
    let cookie = session_cookie(&visit);
    let token = masked_token(&visit);

    let logged_out = app
        .clone()
        .oneshot(post_with("/logout", &cookie, &token))
        .await
        .expect("POST /logout");
    assert_status(&logged_out, 200);
    assert_eq!(raw(&masked_token(&logged_out)), raw(&token));

    let still_valid = app
        .clone()
        .oneshot(post_with("/submit", &cookie, &token))
        .await
        .expect("POST /submit");
    assert_status(&still_valid, 200);
}

#[derive(Clone)]
#[cfg(feature = "orm")]
struct LogoutRaceState {
    db_store: Arc<RuniqueSessionStore>,
    request_entered: Arc<tokio::sync::Notify>,
    release_request: Arc<tokio::sync::Notify>,
}

#[cfg(feature = "orm")]
async fn login_with_db(State(state): State<LogoutRaceState>, session: Session) -> &'static str {
    login(
        &session,
        &test_user(pk(42), "setsuna", false),
        Some(&state.db_store),
        false,
    )
    .await
    .expect("login");
    "logged in"
}

#[cfg(feature = "orm")]
async fn stale_authenticated_request(
    State(state): State<LogoutRaceState>,
    session: Session,
) -> &'static str {
    session
        .insert("concurrent_change", "captured before logout")
        .await
        .expect("modify session");
    state.request_entered.notify_one();
    state.release_request.notified().await;
    "stale request completed"
}

#[cfg(feature = "orm")]
async fn logout_with_db(State(state): State<LogoutRaceState>, session: Session) -> &'static str {
    logout(&session, Some(&state.db_store))
        .await
        .expect("logout");
    "logged out"
}

#[cfg(feature = "orm")]
async fn whoami(session: Session) -> &'static str {
    if runique::auth::session::is_authenticated(&session).await {
        "authenticated"
    } else {
        "anonymous"
    }
}

#[cfg(feature = "orm")]
async fn logout_race_app(
    engine: Arc<runique::engine::RuniqueEngine>,
    state: LogoutRaceState,
    store: CleaningMemoryStore,
) -> Router {
    Router::new()
        .route("/login", get(login_with_db))
        .route("/stale", get(stale_authenticated_request))
        .route("/logout", post(logout_with_db))
        .route("/whoami", get(whoami))
        .with_state(state)
        .layer(middleware::from_fn_with_state(engine, csrf_middleware))
        .layer(SessionManagerLayer::new(store).with_secure(false))
}

#[cfg(feature = "orm")]
fn cookie_pair(resp: &Response<Body>) -> String {
    resp.headers()
        .get("set-cookie")
        .expect("response should set a session cookie")
        .to_str()
        .expect("ASCII cookie")
        .split(';')
        .next()
        .expect("cookie pair")
        .to_string()
}

#[cfg(feature = "orm")]
fn request_with_cookie(method: &str, uri: &str, cookie: &str) -> Request<Body> {
    Request::builder()
        .method(method)
        .uri(uri)
        .header("cookie", cookie)
        .body(Body::empty())
        .expect("request")
}

#[cfg(feature = "orm")]
#[tokio::test]
async fn concurrent_stale_response_cannot_restore_old_authenticated_cookie_after_logout() {
    let conn = crate::helpers::db::fresh_db_with_schema(super::test_session_db::SESSIONS_DDL).await;
    let db_store = Arc::new(RuniqueSessionStore::new(runique::db::ADb::from_connection(
        conn,
    )));
    let request_entered = Arc::new(tokio::sync::Notify::new());
    let release_request = Arc::new(tokio::sync::Notify::new());
    let state = LogoutRaceState {
        db_store: db_store.clone(),
        request_entered: request_entered.clone(),
        release_request: release_request.clone(),
    };
    let memory = CleaningMemoryStore::default().with_db_fallback(db_store.clone());
    let app = logout_race_app(
        crate::helpers::server::build_engine().await,
        state.clone(),
        memory,
    )
    .await;

    let logged_in = app
        .clone()
        .oneshot(Request::get("/login").body(Body::empty()).unwrap())
        .await
        .expect("login request");
    assert_status(&logged_in, 200);
    let old_cookie = cookie_pair(&logged_in);
    let csrf = masked_token(&logged_in);

    let stale_app = app.clone();
    let stale_cookie = old_cookie.clone();
    let stale_request = tokio::spawn(async move {
        stale_app
            .oneshot(request_with_cookie("GET", "/stale", &stale_cookie))
            .await
            .expect("stale request")
    });
    request_entered.notified().await;

    let logout_response = app
        .clone()
        .oneshot(post_with("/logout", &old_cookie, &csrf))
        .await
        .expect("logout request");
    assert_status(&logout_response, 200);
    let new_cookie = cookie_pair(&logout_response);
    assert_ne!(old_cookie, new_cookie, "logout rotates the cookie");

    release_request.notify_one();
    let stale_response = stale_request.await.expect("stale request task");
    assert_status(&stale_response, 200);
    assert_eq!(
        cookie_pair(&stale_response),
        old_cookie,
        "the late response sends the old cookie after logout"
    );

    let old_cookie_same_process = app
        .clone()
        .oneshot(request_with_cookie("GET", "/whoami", &old_cookie))
        .await
        .expect("old cookie request in the same process");
    assert_eq!(
        axum::body::to_bytes(old_cookie_same_process.into_body(), usize::MAX)
            .await
            .unwrap(),
        "anonymous",
        "the in-memory cache must not accept a session revoked in the database"
    );

    // A fresh in-memory store models a server restart: only the DB backup can
    // restore the cookie. The old cookie should have been revoked by logout.
    let fresh_memory = CleaningMemoryStore::default().with_db_fallback(db_store.clone());
    let after_restart = logout_race_app(
        crate::helpers::server::build_engine().await,
        state,
        fresh_memory,
    )
    .await;

    let old_cookie_whoami = after_restart
        .clone()
        .oneshot(request_with_cookie("GET", "/whoami", &old_cookie))
        .await
        .expect("old cookie request");
    assert_eq!(
        axum::body::to_bytes(old_cookie_whoami.into_body(), usize::MAX)
            .await
            .unwrap(),
        "anonymous",
        "the old cookie must remain invalid after a restart"
    );

    let new_cookie_whoami = after_restart
        .clone()
        .oneshot(request_with_cookie("GET", "/whoami", &new_cookie))
        .await
        .expect("new cookie request");
    assert_eq!(
        axum::body::to_bytes(new_cookie_whoami.into_body(), usize::MAX)
            .await
            .unwrap(),
        "anonymous"
    );

    let relogged = after_restart
        .clone()
        .oneshot(request_with_cookie("GET", "/login", &new_cookie))
        .await
        .expect("login with the new anonymous cookie");
    assert_status(&relogged, 200);
    let relogged_cookie = cookie_pair(&relogged);
    let authenticated_again = after_restart
        .oneshot(request_with_cookie("GET", "/whoami", &relogged_cookie))
        .await
        .expect("newly authenticated cookie request");
    assert_eq!(
        axum::body::to_bytes(authenticated_again.into_body(), usize::MAX)
            .await
            .unwrap(),
        "authenticated",
        "a fresh login must still create a valid session"
    );
}
