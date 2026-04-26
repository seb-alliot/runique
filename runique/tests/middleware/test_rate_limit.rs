use crate::helpers::{assert::assert_status, request};
use axum::{Router, routing::get};
use runique::middleware::rate_limit::{RateLimiter, rate_limit_middleware};
use std::sync::Arc;

#[test]
fn test_new_allows_first_request() {
    let limiter = RateLimiter::new().max_requests(5).retry_after(60);
    assert!(limiter.is_allowed("127.0.0.1"));
}

#[test]
fn test_allows_up_to_max() {
    let limiter = RateLimiter::new().max_requests(3).retry_after(60);
    assert!(limiter.is_allowed("ip1"));
    assert!(limiter.is_allowed("ip1"));
    assert!(limiter.is_allowed("ip1"));
    // 4e requête dépasse la limite
    assert!(!limiter.is_allowed("ip1"));
}

#[test]
fn test_different_keys_independent() {
    let limiter = RateLimiter::new().max_requests(1).retry_after(60);
    assert!(limiter.is_allowed("ip-a"));
    assert!(limiter.is_allowed("ip-b")); // clé différente → indépendante
    assert!(!limiter.is_allowed("ip-a")); // ip-a est épuisée
}

#[test]
fn test_window_reset() {
    // Fenêtre de 0 secondes : chaque appel repart d'une nouvelle fenêtre
    let limiter = RateLimiter::new().max_requests(1).retry_after(0);
    assert!(limiter.is_allowed("ip"));
    // La fenêtre est expirée dès le prochain appel
    std::thread::sleep(std::time::Duration::from_millis(10));
    assert!(limiter.is_allowed("ip")); // nouvelle fenêtre → autorisé
}

#[test]
fn test_clone_shares_state() {
    let limiter = RateLimiter::new().max_requests(2).retry_after(60);
    let clone = limiter.clone();
    assert!(limiter.is_allowed("ip"));
    assert!(clone.is_allowed("ip")); // le clone partage le store (Arc)
    // 3e appel sur l'original doit être refusé
    assert!(!limiter.is_allowed("ip"));
}

#[test]
fn test_max_requests_field() {
    let limiter = RateLimiter::new().max_requests(42).retry_after(30);
    assert_eq!(limiter.max_requests, 42);
    assert_eq!(limiter.window.as_secs(), 30);
}

#[test]
fn test_default_values() {
    let limiter = RateLimiter::new();
    assert_eq!(limiter.max_requests, 60);
    assert_eq!(limiter.window.as_secs(), 60);
}

#[test]
fn test_retry_after_zero_when_unknown_key() {
    let limiter = RateLimiter::new().max_requests(5).retry_after(60);
    assert_eq!(limiter.retry_after_secs("unknown"), 0);
}

#[test]
fn test_retry_after_nonzero_after_limit_exceeded() {
    let limiter = RateLimiter::new().max_requests(1).retry_after(60);
    let _ = limiter.is_allowed("ip");
    let _ = limiter.is_allowed("ip"); // dépasse la limite
    assert!(limiter.retry_after_secs("ip") > 0);
}

// ── rate_limit_middleware — intégration HTTP ──────────────────────────────────

fn rate_app(max: u32, retry_after: u64) -> Router {
    let limiter = Arc::new(
        RateLimiter::new()
            .max_requests(max)
            .retry_after(retry_after),
    );
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
    let limiter = Arc::new(RateLimiter::new().max_requests(2).retry_after(60));
    let app = Router::new().route("/", get(|| async { "ok" })).layer(
        axum::middleware::from_fn_with_state(limiter.clone(), rate_limit_middleware),
    );

    // Épuise la limite
    let _ = limiter.is_allowed("unknown");
    let _ = limiter.is_allowed("unknown");

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
async fn test_middleware_429_has_retry_after_header() {
    let limiter = Arc::new(RateLimiter::new().max_requests(1).retry_after(60));
    let app = Router::new().route("/", get(|| async { "ok" })).layer(
        axum::middleware::from_fn_with_state(limiter.clone(), rate_limit_middleware),
    );

    let _ = limiter.is_allowed("unknown");
    let _ = limiter.is_allowed("unknown");

    let resp = request::get(app, "/").await;
    assert_status(&resp, 429);
    assert!(resp.headers().contains_key("retry-after"));
}

#[tokio::test]
async fn test_spawn_cleanup_no_panic() {
    let limiter = RateLimiter::new().max_requests(10).retry_after(60);
    limiter.spawn_cleanup(tokio::time::Duration::from_secs(3600));
    // Pas de panique = OK
}

// ── route_layer — pattern utilisé dans url.rs ─────────────────────────────────

/// Vérifie que route_layer avec rate_limit_middleware bloque après N requêtes.
/// Reproduit exactement le câblage de url.rs pour /upload-image.
#[tokio::test]
async fn test_route_layer_blocks_after_limit() {
    let limiter = Arc::new(RateLimiter::new().max_requests(5).retry_after(60));

    let app = Router::new()
        .route("/upload-image", get(|| async { "ok" }))
        .route_layer(axum::middleware::from_fn_with_state(
            limiter,
            rate_limit_middleware,
        ));

    // Les 5 premières requêtes passent
    for _ in 0..5 {
        let resp = request::get(app.clone(), "/upload-image").await;
        assert_status(&resp, 200);
    }

    // La 6e est bloquée
    let resp = request::get(app.clone(), "/upload-image").await;
    assert_status(&resp, 429);
    assert!(resp.headers().contains_key("retry-after"));
}

/// Vérifie que route_layer n'affecte pas les autres routes du même routeur.
#[tokio::test]
async fn test_route_layer_does_not_affect_other_routes() {
    let limiter = Arc::new(RateLimiter::new().max_requests(1).retry_after(60));

    let app = Router::new()
        .route("/upload-image", get(|| async { "upload" }))
        .route_layer(axum::middleware::from_fn_with_state(
            limiter,
            rate_limit_middleware,
        ))
        .route("/autre", get(|| async { "autre" }));

    // Épuise la limite sur /upload-image
    request::get(app.clone(), "/upload-image").await;
    let bloque = request::get(app.clone(), "/upload-image").await;
    assert_status(&bloque, 429);

    // /autre n'est pas affecté
    let autre = request::get(app.clone(), "/autre").await;
    assert_status(&autre, 200);
}
