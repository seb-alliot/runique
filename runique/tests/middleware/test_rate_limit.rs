use crate::helpers::{assert::assert_status, request};
use axum::{Router, routing::get};
use runique::middleware::rate_limit::{RateLimiter, rate_limit_middleware};
use serial_test::serial;
use std::sync::Arc;

#[test]
fn test_new_allows_first_request() {
    let limiter = RateLimiter::new(5, 60);
    assert!(limiter.is_allowed("127.0.0.1"));
}

#[test]
fn test_allows_up_to_max() {
    let limiter = RateLimiter::new(3, 60);
    assert!(limiter.is_allowed("ip1"));
    assert!(limiter.is_allowed("ip1"));
    assert!(limiter.is_allowed("ip1"));
    // 4e requête dépasse la limite
    assert!(!limiter.is_allowed("ip1"));
}

#[test]
fn test_different_keys_independent() {
    let limiter = RateLimiter::new(1, 60);
    assert!(limiter.is_allowed("ip-a"));
    assert!(limiter.is_allowed("ip-b")); // clé différente → indépendante
    assert!(!limiter.is_allowed("ip-a")); // ip-a est épuisée
}

#[test]
fn test_window_reset() {
    // Fenêtre de 0 secondes : chaque appel repart d'une nouvelle fenêtre
    let limiter = RateLimiter::new(1, 0);
    assert!(limiter.is_allowed("ip"));
    // La fenêtre est expirée dès le prochain appel
    std::thread::sleep(std::time::Duration::from_millis(10));
    assert!(limiter.is_allowed("ip")); // nouvelle fenêtre → autorisé
}

#[test]
fn test_clone_shares_state() {
    let limiter = RateLimiter::new(2, 60);
    let clone = limiter.clone();
    assert!(limiter.is_allowed("ip"));
    assert!(clone.is_allowed("ip")); // le clone partage le store (Arc)
    // 3e appel sur l'original doit être refusé
    assert!(!limiter.is_allowed("ip"));
}

#[test]
fn test_max_requests_field() {
    let limiter = RateLimiter::new(42, 30);
    assert_eq!(limiter.max_requests, 42);
    assert_eq!(limiter.window.as_secs(), 30);
}

#[test]
#[serial]
fn test_from_env_defaults() {
    unsafe {
        std::env::remove_var("RUNIQUE_RATE_LIMIT_REQUESTS");
        std::env::remove_var("RUNIQUE_RATE_LIMIT_WINDOW_SECS");
    }
    let limiter = RateLimiter::from_env();
    assert_eq!(limiter.max_requests, 60);
    assert_eq!(limiter.window.as_secs(), 60);
}

#[test]
#[serial]
fn test_from_env_custom() {
    unsafe {
        std::env::set_var("RUNIQUE_RATE_LIMIT_REQUESTS", "10");
        std::env::set_var("RUNIQUE_RATE_LIMIT_WINDOW_SECS", "120");
    }
    let limiter = RateLimiter::from_env();
    assert_eq!(limiter.max_requests, 10);
    assert_eq!(limiter.window.as_secs(), 120);
    unsafe {
        std::env::remove_var("RUNIQUE_RATE_LIMIT_REQUESTS");
        std::env::remove_var("RUNIQUE_RATE_LIMIT_WINDOW_SECS");
    }
}

// ── rate_limit_middleware — intégration HTTP ──────────────────────────────────

fn rate_app(max: u32, window_secs: u64) -> Router {
    let limiter = Arc::new(RateLimiter::new(max, window_secs));
    Router::new()
        .route("/", get(|| async { "ok" }))
        .layer(axum::middleware::from_fn_with_state(
            limiter,
            rate_limit_middleware,
        ))
}

#[tokio::test]
async fn test_middleware_allows_under_limit() {
    let app = rate_app(5, 60);
    let resp = request::get(app, "/").await;
    assert_status(&resp, 200);
}

#[tokio::test]
async fn test_middleware_blocks_over_limit() {
    let limiter = Arc::new(RateLimiter::new(2, 60));
    let app = Router::new().route("/", get(|| async { "ok" })).layer(
        axum::middleware::from_fn_with_state(limiter.clone(), rate_limit_middleware),
    );

    // Épuise la limite
    limiter.is_allowed("unknown");
    limiter.is_allowed("unknown");

    let resp = request::get(app, "/").await;
    assert_status(&resp, 429);
}

#[tokio::test]
async fn test_middleware_extract_ip_x_forwarded_for() {
    use axum::body::Body;
    use axum::http::Request;
    use tower::ServiceExt;

    let app = rate_app(100, 60);
    let req = Request::builder()
        .uri("/")
        .header("x-forwarded-for", "1.2.3.4, 5.6.7.8")
        .body(Body::empty())
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), 200);
}

#[tokio::test]
async fn test_middleware_extract_ip_x_real_ip() {
    use axum::body::Body;
    use axum::http::Request;
    use tower::ServiceExt;

    let app = rate_app(100, 60);
    let req = Request::builder()
        .uri("/")
        .header("x-real-ip", "9.9.9.9")
        .body(Body::empty())
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), 200);
}

#[tokio::test]
async fn test_middleware_no_ip_header_uses_unknown() {
    let app = rate_app(100, 60);
    let resp = request::get(app, "/").await;
    assert_status(&resp, 200);
}

#[tokio::test]
async fn test_spawn_cleanup_no_panic() {
    let limiter = RateLimiter::new(10, 60);
    limiter.spawn_cleanup(tokio::time::Duration::from_secs(3600));
    // Pas de panique = OK
}
