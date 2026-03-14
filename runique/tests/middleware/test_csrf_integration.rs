//! Integration tests for the CSRF middleware.
//!
//! Two modes:
//!   - `oneshot()` tests: rebuild a fresh Router per test (fast, isolated)
//!   - shared server tests: reuse the persistent server from `helpers::server`
//!     which allows multi-request flows with real cookie/session persistence.

use crate::helpers::{
    assert::{assert_has_header, assert_status},
    request,
    server::{self, build_default_router, build_engine, test_client, test_server_addr},
};

// ── oneshot helpers ────────────────────────────────────────────────────────────

async fn fresh_app() -> axum::Router {
    build_default_router(build_engine().await)
}

// ── oneshot tests (isolated, no session persistence) ──────────────────────────

#[tokio::test]
async fn test_csrf_get_retourne_200_et_header_token() {
    let resp = request::get(fresh_app().await, "/").await;
    assert_status(&resp, 200);
    assert_has_header(&resp, "x-csrf-token");
}

#[tokio::test]
async fn test_csrf_post_sans_header_sans_content_type_retourne_403() {
    // POST sans header X-CSRF-Token et sans Content-Type form : désormais bloqué (403).
    // Les soumissions HTML (urlencoded/multipart) passent toujours via Prisme.
    let resp = request::post(fresh_app().await, "/submit").await;
    assert_status(&resp, 403);
}

#[tokio::test]
async fn test_csrf_post_avec_token_invalide_retourne_403() {
    let resp = request::post_with_header(
        fresh_app().await,
        "/submit",
        "X-CSRF-Token",
        "token_completement_invalide_!!!",
    )
    .await;
    assert_status(&resp, 403);
}

#[tokio::test]
async fn test_csrf_delete_sans_header_sans_content_type_retourne_403() {
    // DELETE sans header X-CSRF-Token et sans Content-Type form : désormais bloqué (403).
    let resp = request::delete(fresh_app().await, "/delete").await;
    assert_status(&resp, 403);
}

#[tokio::test]
async fn test_csrf_delete_avec_token_invalide_retourne_403() {
    let resp = request::delete_with_header(
        fresh_app().await,
        "/delete",
        "X-CSRF-Token",
        "faux_token_base64==",
    )
    .await;
    assert_status(&resp, 403);
}

// ── shared server tests (cookie persistence across requests) ──────────────────

/// Full roundtrip: GET to obtain the token → POST with valid token → 200 OK.
/// Requires a persistent session (cookies), only possible with a real server.
#[tokio::test]
async fn test_csrf_roundtrip_get_then_post_valide() {
    let addr = test_server_addr();
    let client = test_client();

    // Step 1: GET — the server sets the session cookie + returns X-CSRF-Token
    let get_resp = client
        .get(format!("http://{}/", addr))
        .send()
        .await
        .expect("GET /");

    assert_eq!(get_resp.status(), 200);
    let token = server::extract_header(&get_resp, "x-csrf-token");

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

    let stolen_token = server::extract_header(&get_resp, "x-csrf-token");

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
