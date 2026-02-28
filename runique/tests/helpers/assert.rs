//! Assertions custom pour les réponses HTTP axum.
//!
//! # Exemple
//! ```rust
//! use crate::helpers::assert::{assert_status, assert_has_header, assert_body_str};
//!
//! assert_status(&resp, 200);
//! assert_has_header(&resp, "x-csrf-token");
//! assert_body_str(resp, "ok").await;
//! ```

use axum::{body::Body, response::Response};

// ── Statut ────────────────────────────────────────────────────────────────────

/// Vérifie le code de statut HTTP (ex: `assert_status(&resp, 200)`).
pub fn assert_status(resp: &Response<Body>, expected: u16) {
    assert_eq!(
        resp.status().as_u16(),
        expected,
        "Statut attendu {expected}, reçu {}",
        resp.status()
    );
}

/// Vérifie que le statut est une redirection (3xx).
pub fn assert_is_redirect(resp: &Response<Body>) {
    assert!(
        resp.status().is_redirection(),
        "Redirection attendue, reçu {}",
        resp.status()
    );
}

// ── Headers ───────────────────────────────────────────────────────────────────

/// Vérifie qu'un header est présent dans la réponse.
pub fn assert_has_header(resp: &Response<Body>, key: &str) {
    assert!(
        resp.headers().contains_key(key),
        "Header '{key}' absent de la réponse"
    );
}

/// Vérifie la valeur exacte d'un header.
pub fn assert_header_eq(resp: &Response<Body>, key: &str, expected: &str) {
    let value = resp
        .headers()
        .get(key)
        .unwrap_or_else(|| panic!("Header '{key}' absent"))
        .to_str()
        .expect("valeur de header non-UTF8");
    assert_eq!(value, expected, "Header '{key}' — valeur inattendue");
}

// ── Redirections ──────────────────────────────────────────────────────────────

/// Vérifie que la réponse est une redirection vers `location`.
pub fn assert_redirect(resp: &Response<Body>, location: &str) {
    assert_is_redirect(resp);
    assert_header_eq(resp, "location", location);
}

// ── Body ──────────────────────────────────────────────────────────────────────

/// Lit le body d'une réponse axum comme `String` UTF-8.
pub async fn body_str(resp: Response) -> String {
    let bytes = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .expect("lecture du body échouée");
    String::from_utf8(bytes.to_vec()).expect("body non-UTF8")
}

/// Vérifie que le body texte d'une réponse axum est égal à `expected`.
///
/// # Exemple
/// ```rust
/// assert_body_str(resp, "anonymous").await;
/// ```
pub async fn assert_body_str(resp: Response, expected: &str) {
    let body = body_str(resp).await;
    assert_eq!(body, expected, "Body inattendu");
}
