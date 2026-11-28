// Tests pour RequestExtensions — builder pattern + inject

use crate::helpers::pk::pk;
use axum::{body::Body, http::Request};
use runique::auth::current_user::CurrentUser;
use runique::http::RequestExtensions;
use runique::security::csp_nonce::CspNonce;
use runique::security::csrf::CsrfToken;

// ── new / default ────────────────────────────────────────────────

#[test]
fn test_new_all_fields_none() {
    let ext = RequestExtensions::new();
    assert!(ext.engine.is_none());
    assert!(ext.csrf_token.is_none());
    assert!(ext.csp_nonce.is_none());
    assert!(ext.current_user.is_none());
}

#[test]
fn test_default_equivalent_to_new() {
    let ext = RequestExtensions::default();
    assert!(ext.engine.is_none());
    assert!(ext.csrf_token.is_none());
}

// ── with_csrf_token ──────────────────────────────────────────────

#[test]
fn test_with_csrf_token() {
    let token = CsrfToken("test_token".to_string());
    let ext = RequestExtensions::new().with_csrf_token(token);
    assert!(ext.csrf_token.is_some());
}

// ── with_csp_nonce ───────────────────────────────────────────────

#[test]
fn test_with_csp_nonce() {
    let nonce = CspNonce::generate();
    let ext = RequestExtensions::new().with_csp_nonce(nonce);
    assert!(ext.csp_nonce.is_some());
}

// ── with_current_user ────────────────────────────────────────────

#[test]
fn test_with_current_user() {
    let user = CurrentUser {
        id: pk(1),
        username: "alice".to_string(),
        rang: runique::auth::permissions::rank::Rang::Lambda,
        groupes: vec![],
        droits: Vec::new(),
    };
    let ext = RequestExtensions::new().with_current_user(user);
    assert!(ext.current_user.is_some());
}

// ── inject_request ───────────────────────────────────────────────

#[test]
fn test_inject_request_csrf_token() {
    let token = CsrfToken("inject_test".to_string());
    let ext = RequestExtensions::new().with_csrf_token(token);

    let mut req = Request::builder().uri("/").body(Body::empty()).unwrap();

    ext.inject_request(&mut req);

    assert!(req.extensions().get::<CsrfToken>().is_some());
}

#[test]
fn test_inject_request_csp_nonce() {
    let nonce = CspNonce::generate();
    let ext = RequestExtensions::new().with_csp_nonce(nonce);

    let mut req = Request::builder().uri("/").body(Body::empty()).unwrap();

    ext.inject_request(&mut req);

    assert!(req.extensions().get::<CspNonce>().is_some());
}

#[test]
fn test_inject_request_current_user() {
    let user = CurrentUser {
        id: pk(42),
        username: "bob".to_string(),
        rang: runique::auth::permissions::rank::Rang::Lambda,
        groupes: vec![],
        droits: Vec::new(),
    };
    let ext = RequestExtensions::new().with_current_user(user);

    let mut req = Request::builder().uri("/").body(Body::empty()).unwrap();

    ext.inject_request(&mut req);

    let injected = req.extensions().get::<CurrentUser>().unwrap();
    assert_eq!(injected.username, "bob");
}
