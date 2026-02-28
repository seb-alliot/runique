//! Integration tests for the CSRF middleware.
//!
//! Two modes:
//!   - `oneshot()` tests: rebuild a fresh Router per test (fast, isolated)
//!   - shared server tests: reuse the persistent server from `helpers::server`
//!     which allows multi-request flows with real cookie/session persistence.

use axum::{
    body::Body,
    http::{Method, Request, StatusCode},
};
use tower::ServiceExt;

use crate::helpers::server::{build_engine, build_default_router, test_client, test_server_addr};

// ── oneshot helpers ────────────────────────────────────────────────────────────

async fn fresh_app() -> axum::Router {
    let engine = build_engine().await;
    build_default_router(engine)
}

// ── oneshot tests (isolated, no session persistence) ──────────────────────────

#[tokio::test]
async fn test_csrf_get_retourne_200_et_header_token() {
    let app = fresh_app().await;

    let req = Request::builder()
        .method(Method::GET)
        .uri("/")
        .body(Body::empty())
        .unwrap();

    let resp = app.oneshot(req).await.unwrap();

    assert_eq!(resp.status(), StatusCode::OK);
    assert!(
        resp.headers().contains_key("x-csrf-token"),
        "X-CSRF-Token must be present in GET response"
    );
}

#[tokio::test]
async fn test_csrf_post_sans_header_passe() {
    let app = fresh_app().await;

    let req = Request::builder()
        .method(Method::POST)
        .uri("/submit")
        .body(Body::empty())
        .unwrap();

    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_csrf_post_avec_token_invalide_retourne_403() {
    let app = fresh_app().await;

    let req = Request::builder()
        .method(Method::POST)
        .uri("/submit")
        .header("X-CSRF-Token", "token_completement_invalide_!!!")
        .body(Body::empty())
        .unwrap();

    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn test_csrf_delete_sans_header_passe() {
    let app = fresh_app().await;

    let req = Request::builder()
        .method(Method::DELETE)
        .uri("/delete")
        .body(Body::empty())
        .unwrap();

    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_csrf_delete_avec_token_invalide_retourne_403() {
    let app = fresh_app().await;

    let req = Request::builder()
        .method(Method::DELETE)
        .uri("/delete")
        .header("X-CSRF-Token", "faux_token_base64==")
        .body(Body::empty())
        .unwrap();

    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);
}

// ── shared server tests (cookie persistence across requests) ──────────────────

/// Full roundtrip: GET to obtain the token → POST with valid token → 200 OK.
/// Requires a persistent session (cookies), only possible with a real server.
#[tokio::test]
async fn test_csrf_roundtrip_get_then_post_valide() {
    let addr = test_server_addr();
    let client = test_client(); // cookie_store enabled

    // Step 1: GET — the server sets the session cookie + returns X-CSRF-Token
    let get_resp = client
        .get(format!("http://{}/", addr))
        .send()
        .await
        .expect("GET /");

    assert_eq!(get_resp.status(), 200);

    let token = get_resp
        .headers()
        .get("x-csrf-token")
        .expect("X-CSRF-Token header missing")
        .to_str()
        .expect("header to str")
        .to_string();

    // Step 2: POST with the token obtained in step 1 — same session via cookie
    let post_resp = client
        .post(format!("http://{}/submit", addr))
        .header("X-CSRF-Token", &token)
        .send()
        .await
        .expect("POST /submit");

    assert_eq!(
        post_resp.status(),
        200,
        "POST with valid CSRF token should return 200"
    );
}

/// Confirms that a second client (different session) cannot reuse a stolen token.
#[tokio::test]
async fn test_csrf_token_vol_autre_session_retourne_403() {
    let addr = test_server_addr();

    // Client A: obtains a token for its own session
    let client_a = test_client();
    let get_resp = client_a
        .get(format!("http://{}/", addr))
        .send()
        .await
        .expect("GET / client A");

    let stolen_token = get_resp
        .headers()
        .get("x-csrf-token")
        .expect("token header")
        .to_str()
        .expect("str")
        .to_string();

    // Client B: different session, tries to use client A's token
    let client_b = test_client();
    let post_resp = client_b
        .post(format!("http://{}/submit", addr))
        .header("X-CSRF-Token", &stolen_token)
        .send()
        .await
        .expect("POST /submit client B");

    assert_eq!(
        post_resp.status(),
        403,
        "Token from another session must be rejected"
    );
}
