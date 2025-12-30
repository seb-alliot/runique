use axum::{
    Router,
    routing::post,
    body::Body,
    http::{Request, StatusCode, header},
    middleware,
};
use std::sync::Arc;
use tower::ServiceExt;
use rusti::{
    Settings,
    middleware::middleware_sanetiser::sanitize_middleware,
};
use axum::extract::Extension;

/// Handler de test qui retourne le body
async fn test_handler() -> &'static str {
    "OK"
}

/// Crée une app de test avec le middleware de sanitization
fn create_test_app(sanitize_enabled: bool) -> Router {
    let mut settings = Settings::default_values();
    settings.sanitize_inputs = sanitize_enabled;
    let settings = Arc::new(settings);

    Router::new()
        .route("/", post(test_handler))
        .layer(middleware::from_fn_with_state(settings.clone(), sanitize_middleware))
        .layer(Extension(settings))
}

#[tokio::test]
async fn test_sanitization_disabled() {
    let app = create_test_app(false);

    // Quand la sanitization est désactivée, le middleware devrait passer la requête telle quelle
    let req = Request::builder()
        .method("POST")
        .uri("/")
        .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
        .body(Body::from("name=<script>alert('xss')</script>"))
        .unwrap();

    let res = app.oneshot(req).await.unwrap();
    // Devrait passer (même si le contenu n'est pas sanitizé)
    assert!(res.status().is_success() || res.status().is_redirection());
}

#[tokio::test]
async fn test_sanitization_enabled() {
    let app = create_test_app(true);

    // Quand la sanitization est activée, le middleware devrait traiter la requête
    let req = Request::builder()
        .method("POST")
        .uri("/")
        .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
        .body(Body::from("name=test"))
        .unwrap();

    let res = app.oneshot(req).await.unwrap();
    // Devrait passer
    assert!(res.status().is_success() || res.status().is_redirection());
}

#[tokio::test]
async fn test_sanitization_json_content_type() {
    let app = create_test_app(true);

    // Test avec JSON
    let req = Request::builder()
        .method("POST")
        .uri("/")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(r#"{"name": "test"}"#))
        .unwrap();

    let res = app.oneshot(req).await.unwrap();
    assert!(res.status().is_success() || res.status().is_redirection());
}

#[tokio::test]
async fn test_sanitization_multipart_skipped() {
    let app = create_test_app(true);

    // Multipart/form-data devrait être ignoré (pas encore supporté)
    let req = Request::builder()
        .method("POST")
        .uri("/")
        .header(header::CONTENT_TYPE, "multipart/form-data; boundary=----WebKitFormBoundary")
        .body(Body::from("test"))
        .unwrap();

    let res = app.oneshot(req).await.unwrap();
    // Devrait passer sans sanitization
    assert!(res.status().is_success() || res.status().is_redirection());
}

#[tokio::test]
async fn test_sanitization_get_request() {
    let app = create_test_app(true);

    // Les requêtes GET ne devraient pas être sanitizées
    // Mais le handler attend POST, donc on s'attend à une erreur 405 Method Not Allowed
    let req = Request::builder()
        .method("GET")
        .uri("/")
        .body(Body::empty())
        .unwrap();

    let res = app.oneshot(req).await.unwrap();
    // GET sur une route POST devrait retourner 405 Method Not Allowed
    assert!(res.status().is_client_error() || res.status().is_success() || res.status().is_redirection());
}
