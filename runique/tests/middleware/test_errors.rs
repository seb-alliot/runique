// Tests pour error_handler_middleware

use axum::{http::StatusCode, middleware, routing::get, Extension, Router};
use http_body_util::BodyExt;
use runique::{
    config::app::RuniqueConfig,
    middleware::errors::error::{error_handler_middleware, RequestInfoHelper},
};
use std::{collections::HashMap, sync::Arc};

use crate::helpers::{request, server::build_engine};

// ═══════════════════════════════════════════════════════════════
// Builders locaux
// ═══════════════════════════════════════════════════════════════

/// Router minimal avec error_handler_middleware en mode production (debug=false).
async fn build_error_app() -> Router {
    let engine = build_engine().await;
    let tera = engine.tera.clone();
    let config = Arc::new(engine.config.clone()); // debug=false par défaut

    Router::new()
        .route("/ok", get(|| async { "ok" }))
        .route(
            "/error500",
            get(|| async { StatusCode::INTERNAL_SERVER_ERROR }),
        )
        .layer(middleware::from_fn(error_handler_middleware))
        .layer(Extension(tera))
        .layer(Extension(config))
}

/// Même router mais avec debug=true.
async fn build_debug_error_app() -> Router {
    let engine = build_engine().await;
    let tera = engine.tera.clone();
    let mut config = engine.config.clone();
    config.debug = true;
    let config = Arc::new(config);

    Router::new()
        .route("/ok", get(|| async { "ok" }))
        .route(
            "/error500",
            get(|| async { StatusCode::INTERNAL_SERVER_ERROR }),
        )
        .layer(middleware::from_fn(error_handler_middleware))
        .layer(Extension(tera))
        .layer(Extension(config))
}

// ═══════════════════════════════════════════════════════════════
// Tests struct
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_request_info_helper_struct() {
    let mut headers = HashMap::new();
    headers.insert("x-test-header".to_string(), "valeur".to_string());
    let helper = RequestInfoHelper {
        method: "GET".to_string(),
        path: "/".to_string(),
        query: Some("a=1".to_string()),
        headers: headers.clone(),
    };
    assert_eq!(helper.method, "GET");
    assert_eq!(helper.path, "/");
    assert_eq!(helper.query, Some("a=1".to_string()));
    assert_eq!(helper.headers.get("x-test-header").unwrap(), "valeur");
}

#[test]
fn test_request_info_helper_sans_query() {
    let helper = RequestInfoHelper {
        method: "POST".to_string(),
        path: "/submit".to_string(),
        query: None,
        headers: HashMap::new(),
    };
    assert_eq!(helper.method, "POST");
    assert!(helper.query.is_none());
    assert!(helper.headers.is_empty());
}

#[test]
fn test_runique_config_debug_false_par_defaut() {
    let config = RuniqueConfig::default();
    assert!(!config.debug, "debug doit être false par défaut");
}

// ═══════════════════════════════════════════════════════════════
// Tests d'intégration — mode production (debug=false)
// ═══════════════════════════════════════════════════════════════

#[tokio::test]
async fn test_200_passe_sans_modification() {
    let resp = request::get(build_error_app().await, "/ok").await;
    assert_eq!(resp.status(), 200);
}

#[tokio::test]
async fn test_404_retourne_404() {
    let resp = request::get(build_error_app().await, "/route_inexistante").await;
    assert_eq!(resp.status(), 404);
}

#[tokio::test]
async fn test_404_retourne_html() {
    let resp = request::get(build_error_app().await, "/introuvable").await;
    assert_eq!(resp.status(), 404);
    let ct = resp
        .headers()
        .get("content-type")
        .expect("Content-Type absent")
        .to_str()
        .unwrap();
    assert!(ct.contains("text/html"), "attendu text/html, obtenu: {ct}");
}

#[tokio::test]
async fn test_404_body_contient_404() {
    let resp = request::get(build_error_app().await, "/introuvable").await;
    let body = resp.into_body().collect().await.unwrap().to_bytes();
    let html = String::from_utf8_lossy(&body);
    assert!(html.contains("404"), "Le body devrait contenir '404'");
}

#[tokio::test]
async fn test_500_retourne_500() {
    let resp = request::get(build_error_app().await, "/error500").await;
    assert_eq!(resp.status(), 500);
}

#[tokio::test]
async fn test_500_retourne_html() {
    let resp = request::get(build_error_app().await, "/error500").await;
    let ct = resp
        .headers()
        .get("content-type")
        .expect("Content-Type absent")
        .to_str()
        .unwrap();
    assert!(ct.contains("text/html"), "attendu text/html, obtenu: {ct}");
}

#[tokio::test]
async fn test_500_body_contient_500() {
    let resp = request::get(build_error_app().await, "/error500").await;
    let body = resp.into_body().collect().await.unwrap().to_bytes();
    let html = String::from_utf8_lossy(&body);
    assert!(html.contains("500"), "Le body devrait contenir '500'");
}

// ═══════════════════════════════════════════════════════════════
// Tests d'intégration — mode debug (debug=true)
// ═══════════════════════════════════════════════════════════════

#[tokio::test]
async fn test_debug_200_passe_sans_modification() {
    let resp = request::get(build_debug_error_app().await, "/ok").await;
    assert_eq!(resp.status(), 200);
}

#[tokio::test]
async fn test_debug_404_retourne_html() {
    // En mode debug, Tera::default() n'a pas de template "debug" →
    // critical_error_html() est appelé → retourne HTML
    let resp = request::get(build_debug_error_app().await, "/introuvable").await;
    let ct = resp
        .headers()
        .get("content-type")
        .expect("Content-Type absent")
        .to_str()
        .unwrap();
    assert!(ct.contains("text/html"), "attendu text/html en mode debug");
}

#[tokio::test]
async fn test_debug_500_retourne_html() {
    let resp = request::get(build_debug_error_app().await, "/error500").await;
    let ct = resp
        .headers()
        .get("content-type")
        .expect("Content-Type absent")
        .to_str()
        .unwrap();
    assert!(ct.contains("text/html"), "attendu text/html en mode debug");
}

#[tokio::test]
async fn test_debug_body_contient_info_erreur() {
    // Sans template "debug" dans Tera::default(), critical_error_html est appelé
    let resp = request::get(build_debug_error_app().await, "/error500").await;
    let body = resp.into_body().collect().await.unwrap().to_bytes();
    let html = String::from_utf8_lossy(&body);
    assert!(
        html.contains("CRITICAL") || html.contains("Tera") || html.contains("Error"),
        "Le body debug devrait contenir une info d'erreur de rendu"
    );
}
