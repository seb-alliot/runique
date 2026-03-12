//! Builders de requêtes HTTP pour les tests oneshot.
//!
//! Permet d'écrire un test en 2-3 lignes au lieu de 6-8.
//!
//! # Exemple
//! ```rust
//! use crate::helpers::{request, server::build_engine, server::build_default_router};
//!
//! #[tokio::test]
//! async fn mon_test() {
//!     let app = build_default_router(build_engine().await);
//!     let resp = request::get(app, "/").await;
//!     assert_eq!(resp.status(), 200);
//! }
//! ```

use axum::{
    Router,
    body::Body,
    http::{Method, Request, header},
    response::Response,
};
use tower::ServiceExt;

// ── GET ───────────────────────────────────────────────────────────────────────

/// Envoie une requête GET en oneshot sur `uri`.
pub async fn get(app: Router, uri: &str) -> Response {
    let req = Request::builder()
        .method(Method::GET)
        .uri(uri)
        .body(Body::empty())
        .unwrap();
    app.oneshot(req).await.unwrap()
}

/// Envoie une requête GET avec un header supplémentaire.
pub async fn get_with_header(app: Router, uri: &str, hdr: &str, value: &str) -> Response {
    let req = Request::builder()
        .method(Method::GET)
        .uri(uri)
        .header(hdr, value)
        .body(Body::empty())
        .unwrap();
    app.oneshot(req).await.unwrap()
}

// ── POST ──────────────────────────────────────────────────────────────────────

/// Envoie une requête POST vide en oneshot sur `uri`.
pub async fn post(app: Router, uri: &str) -> Response {
    let req = Request::builder()
        .method(Method::POST)
        .uri(uri)
        .body(Body::empty())
        .unwrap();
    app.oneshot(req).await.unwrap()
}

/// Envoie une requête POST avec un header supplémentaire (ex: X-CSRF-Token).
pub async fn post_with_header(app: Router, uri: &str, hdr: &str, value: &str) -> Response {
    let req = Request::builder()
        .method(Method::POST)
        .uri(uri)
        .header(hdr, value)
        .body(Body::empty())
        .unwrap();
    app.oneshot(req).await.unwrap()
}

// ── DELETE ────────────────────────────────────────────────────────────────────

/// Envoie une requête DELETE en oneshot sur `uri`.
pub async fn delete(app: Router, uri: &str) -> Response {
    let req = Request::builder()
        .method(Method::DELETE)
        .uri(uri)
        .body(Body::empty())
        .unwrap();
    app.oneshot(req).await.unwrap()
}

/// Envoie une requête DELETE avec un header supplémentaire.
pub async fn delete_with_header(app: Router, uri: &str, hdr: &str, value: &str) -> Response {
    let req = Request::builder()
        .method(Method::DELETE)
        .uri(uri)
        .header(hdr, value)
        .body(Body::empty())
        .unwrap();
    app.oneshot(req).await.unwrap()
}

// ── Bare request builders (pas de dispatch) ───────────────────────────────────

/// Construit une requête GET avec un header `Host` — utile pour les fonctions
/// qui inspectent les headers directement (ex: `is_localhost()`).
///
/// # Exemple
/// ```rust
/// use crate::helpers::request::build_with_host;
///
/// assert!(is_localhost(&build_with_host("localhost:3000")));
/// assert!(!is_localhost(&build_with_host("evil.com")));
/// ```
pub fn build_with_host(host: &str) -> Request<Body> {
    Request::builder()
        .method(Method::GET)
        .header(header::HOST, host)
        .body(Body::empty())
        .unwrap()
}
