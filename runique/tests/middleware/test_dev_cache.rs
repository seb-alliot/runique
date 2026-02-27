// Tests pour dev_no_cache_middleware (structure)

use axum::http::header;
use axum::http::Request;
use runique::middleware::dev::cache::is_localhost;

#[test]
fn test_is_localhost_true() {
    let req = Request::builder()
        .header(header::HOST, "localhost:3000")
        .body(axum::body::Body::empty())
        .unwrap();
    assert!(is_localhost(&req));
}

#[test]
fn test_is_localhost_127() {
    let req = Request::builder()
        .header(header::HOST, "127.0.0.1:8080")
        .body(axum::body::Body::empty())
        .unwrap();
    assert!(is_localhost(&req));
}

#[test]
fn test_is_localhost_ipv6() {
    let req = Request::builder()
        .header(header::HOST, "[::1]:3000")
        .body(axum::body::Body::empty())
        .unwrap();
    assert!(is_localhost(&req));
}

#[test]
fn test_is_localhost_false() {
    let req = Request::builder()
        .header(header::HOST, "evil.com")
        .body(axum::body::Body::empty())
        .unwrap();
    assert!(!is_localhost(&req));
}
