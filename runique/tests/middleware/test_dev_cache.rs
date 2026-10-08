// Tests pour dev_no_cache_middleware (structure)

use crate::helpers::request::build_with_host;
use runique::middleware::dev::cache::is_localhost;

#[test]
fn test_is_localhost_true() {
    assert!(is_localhost(&build_with_host("localhost:3000")));
}

#[test]
fn test_is_localhost_127() {
    assert!(is_localhost(&build_with_host("127.0.0.1:8080")));
}

#[test]
fn test_is_localhost_ipv6() {
    assert!(is_localhost(&build_with_host("[::1]:3000")));
}

#[test]
fn test_is_localhost_false() {
    assert!(!is_localhost(&build_with_host("evil.com")));
}

// Written from cargo-mutants survivors (2026-10-08): the middleware itself was
// never run — dev/cache.rs:20 (`debug && localhost`).
async fn cache_control(debug: bool, host: &str) -> Option<String> {
    use crate::helpers::server::{build_engine, build_engine_debug};
    use axum::{Router, middleware, routing::get};
    use runique::middleware::dev::cache::dev_no_cache_middleware;
    let engine = if debug {
        build_engine_debug().await
    } else {
        build_engine().await
    };
    let app =
        Router::new()
            .route("/", get(|| async { "ok" }))
            .layer(middleware::from_fn_with_state(
                engine,
                dev_no_cache_middleware,
            ));
    let resp = crate::helpers::request::get_with_header(app, "/", "host", host).await;
    resp.headers()
        .get("cache-control")
        .map(|v| v.to_str().unwrap().to_string())
}

#[tokio::test]
async fn test_no_cache_only_in_debug_on_localhost() {
    let no_store = Some("no-cache, no-store, must-revalidate".to_string());
    assert_eq!(cache_control(true, "localhost:3000").await, no_store);
    assert_eq!(
        cache_control(true, "example.com").await,
        None,
        "debug, but not local"
    );
    assert_eq!(
        cache_control(false, "localhost:3000").await,
        None,
        "local, but production"
    );
}
