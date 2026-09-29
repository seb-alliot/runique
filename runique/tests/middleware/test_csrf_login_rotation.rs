//! The CSRF token changes whenever the privilege level does: at login, like
//! the session id, and at logout.
//!
//! The attack this closes: someone plants an anonymous session on a victim
//! (session fixation) and so knows its CSRF token. `login()` already cycles the
//! session id, but the token used to survive the login, so the attacker could
//! still forge requests the victim's logged-in session would accept.

use crate::helpers::{assert::assert_status, pk::pk, server::build_engine, user::test_user};
use axum::{
    Router,
    body::Body,
    http::{Request, Response},
    middleware,
    routing::{get, post},
};
use runique::{
    auth::session::{login, logout},
    middleware::security::csrf::csrf_middleware,
    utils::crypto::csrf::CsrfToken,
};
use tower::ServiceExt;
use tower_sessions::{MemoryStore, Session, SessionManagerLayer};

async fn log_in(session: Session) -> &'static str {
    let db = runique::db::ADb::from_connection(
        sea_orm::Database::connect("sqlite::memory:")
            .await
            .expect("sqlite::memory: connect"),
    );
    login(
        &session,
        &db,
        &test_user(pk(1), "setsuna", true, false),
        None,
        false,
    )
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
