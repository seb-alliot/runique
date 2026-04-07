// Tests pour RequestExtensions — builder pattern + inject

use axum::{body::Body, http::Request};
use runique::context::RequestExtensions;
use runique::middleware::auth::CurrentUser;
use runique::utils::csp_nonce::CspNonce;
use runique::utils::csrf::CsrfToken;

// ── new / default ────────────────────────────────────────────────

#[test]
fn test_new_all_fields_none() {
    let ext = RequestExtensions::new();
    assert!(ext.engine.is_none());
    assert!(ext.tera.is_none());
    assert!(ext.config.is_none());
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
        id: 1,
        username: "alice".to_string(),
        is_staff: false,
        is_superuser: false,

        groupes: vec![],
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
        id: 42,
        username: "bob".to_string(),
        is_staff: true,
        is_superuser: false,

        groupes: vec![],
    };
    let ext = RequestExtensions::new().with_current_user(user);

    let mut req = Request::builder().uri("/").body(Body::empty()).unwrap();

    ext.inject_request(&mut req);

    let injected = req.extensions().get::<CurrentUser>().unwrap();
    assert_eq!(injected.username, "bob");
}

// ── inject (Parts) ───────────────────────────────────────────────

#[test]
fn test_inject_parts_csrf_token() {
    use axum::http::Request;

    let token = CsrfToken("parts_token".to_string());
    let ext = RequestExtensions::new().with_csrf_token(token);

    let req = Request::builder().uri("/").body(Body::empty()).unwrap();
    let (mut parts, _body) = req.into_parts();

    ext.inject(&mut parts);

    assert!(parts.extensions.get::<CsrfToken>().is_some());
}

#[test]
fn test_inject_empty_does_nothing() {
    let ext = RequestExtensions::new();
    let req = Request::builder().uri("/").body(Body::empty()).unwrap();
    let (mut parts, _) = req.into_parts();

    ext.inject(&mut parts);

    assert!(parts.extensions.get::<CsrfToken>().is_none());
    assert!(parts.extensions.get::<CspNonce>().is_none());
}
