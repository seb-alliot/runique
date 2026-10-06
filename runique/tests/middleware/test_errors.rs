// Tests pour error_handler_middleware

use axum::{Extension, Router, http::StatusCode, middleware, routing::get};
use http_body_util::BodyExt;
use runique::{
    config::app::RuniqueConfig,
    context::template::AppError,
    errors::error::ErrorContext,
    middleware::errors::error::{RequestInfoHelper, error_handler_middleware},
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

async fn bad_query() -> Box<AppError> {
    Box::new(AppError::new(ErrorContext::bad_request(
        "page: invalid digit",
    )))
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

// ═══════════════════════════════════════════════════════════════
// 400 — seulement ceux construits par Runique (`ErrorType::BadRequest`)
// ═══════════════════════════════════════════════════════════════

/// The framework's embedded templates, as `load_internal_templates` registers
/// them: filters first, then `SIMPLE_TEMPLATES` + `ERROR_CORPS` in one batch
/// (`debug.html` includes the `ERROR_CORPS` partials).
fn internal_tera() -> tera::Tera {
    use runique::utils::constante::template::{ERROR_CORPS, SIMPLE_TEMPLATES};
    let mut tera = tera::Tera::default();
    tera.autoescape_on(vec!["html", "xml"]);
    runique::context::tera::static_tera::register_asset_filters(
        &mut tera,
        "/static".to_string(),
        "/media".to_string(),
        "/static".to_string(),
        "/media".to_string(),
        Arc::new(std::sync::RwLock::new(HashMap::new())),
    );
    let templates: Vec<(&str, &str)> = SIMPLE_TEMPLATES
        .iter()
        .chain(ERROR_CORPS.iter())
        .copied()
        .collect();
    tera.add_raw_templates(templates)
        .expect("internal templates");
    tera
}

/// Router with the framework's real templates (400.html, debug.html).
async fn build_templated_error_app(debug: bool) -> Router {
    let engine = build_engine().await;
    let mut config = engine.config.clone();
    config.debug = debug;
    let tera = Arc::new(internal_tera());
    Router::new()
        .route("/bad_query", get(bad_query))
        .route(
            "/api_400",
            get(|| async { (StatusCode::BAD_REQUEST, "{\"error\":\"api\"}") }),
        )
        .layer(middleware::from_fn(error_handler_middleware))
        .layer(Extension(tera))
        .layer(Extension(Arc::new(config)))
}

async fn get_page(debug: bool, path: &str) -> (StatusCode, String) {
    let resp = request::get(build_templated_error_app(debug).await, path).await;
    let status = resp.status();
    let body = resp.into_body().collect().await.unwrap().to_bytes();
    (status, String::from_utf8_lossy(&body).into_owned())
}

#[tokio::test]
async fn test_bad_request_renders_the_400_template() {
    let (status, html) = get_page(false, "/bad_query").await;
    assert_eq!(status, 400);
    assert!(
        html.contains("<h1 id=\"number-error\""),
        "the 400.html template, not the fallback: {html}"
    );
    assert!(
        !html.contains("invalid digit"),
        "the reason stays off the production page"
    );
}

#[tokio::test]
async fn test_debug_bad_request_shows_the_reason() {
    let (status, html) = get_page(true, "/bad_query").await;
    assert_eq!(status, 400, "{html}");
    assert!(html.contains("invalid digit"));
}

#[tokio::test]
async fn test_an_api_400_body_is_left_untouched() {
    for debug in [false, true] {
        let (status, body) = get_page(debug, "/api_400").await;
        assert_eq!(status, 400);
        assert_eq!(body, r#"{"error":"api"}"#);
    }
}
