use axum::extract::Extension;
use axum::{
    body::Body,
    http::{header, Method, Request, StatusCode},
    middleware,
    routing::{get, post},
    Router,
};
use runique::{middleware::csrf::csrf_middleware, Settings};
use std::sync::Arc;
use tower::ServiceExt;
use tower_sessions::{MemoryStore, SessionManagerLayer};

/// Handler GET simple (pas de CSRF requis)
async fn get_handler() -> &'static str {
    "GET OK"
}

/// Handler POST (CSRF requis)
async fn post_handler() -> &'static str {
    "POST OK"
}

/// Crée une app de test avec le middleware CSRF
fn create_test_app() -> Router {
    let settings = Arc::new(Settings::default_values());
    let store = MemoryStore::default();

    let session_layer = SessionManagerLayer::new(store)
        .with_secure(false)
        .with_expiry(tower_sessions::Expiry::OnSessionEnd);

    Router::new()
        .route("/get", get(get_handler))
        .route("/post", post(post_handler))
        .layer(middleware::from_fn(csrf_middleware))
        .layer(Extension(settings))
        .layer(session_layer)
}

#[tokio::test]
async fn test_csrf_get_request_allowed() {
    let app = create_test_app();

    // Les requêtes GET ne nécessitent pas de CSRF
    let req = Request::builder()
        .method(Method::GET)
        .uri("/get")
        .body(Body::empty())
        .unwrap();

    let res = app.oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_csrf_post_without_token_rejected() {
    let app = create_test_app();

    // Les requêtes POST sans token CSRF doivent être rejetées
    // Le middleware redirige vers "/" si le token est invalide
    let req = Request::builder()
        .method(Method::POST)
        .uri("/post")
        .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
        .body(Body::empty())
        .unwrap();

    let res = app.oneshot(req).await.unwrap();
    // Le middleware redirige (302) ou retourne une erreur selon le contexte
    // Vérifions qu'il ne retourne pas 200 OK
    assert_ne!(res.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_csrf_middleware_structure() {
    let app = create_test_app();

    // Test que le middleware est bien configuré et fonctionne
    // Les requêtes GET ne nécessitent pas de CSRF
    let get_req = Request::builder()
        .method(Method::GET)
        .uri("/get")
        .body(Body::empty())
        .unwrap();

    let get_res = app.clone().oneshot(get_req).await.unwrap();
    assert_eq!(get_res.status(), StatusCode::OK);

    // Les requêtes POST sans session/token sont rejetées
    let post_req = Request::builder()
        .method(Method::POST)
        .uri("/post")
        .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
        .body(Body::empty())
        .unwrap();

    let post_res = app.oneshot(post_req).await.unwrap();
    // Devrait être rejeté (redirection ou erreur)
    assert_ne!(post_res.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_csrf_put_requires_token() {
    let app = create_test_app();

    // Les requêtes PUT nécessitent aussi un token CSRF
    let req = Request::builder()
        .method(Method::PUT)
        .uri("/post")
        .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
        .body(Body::empty())
        .unwrap();

    let res = app.oneshot(req).await.unwrap();
    // Le middleware redirige ou retourne une erreur
    assert_ne!(res.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_csrf_delete_requires_token() {
    let app = create_test_app();

    // Les requêtes DELETE nécessitent aussi un token CSRF
    let req = Request::builder()
        .method(Method::DELETE)
        .uri("/post")
        .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
        .body(Body::empty())
        .unwrap();

    let res = app.oneshot(req).await.unwrap();
    // Le middleware redirige ou retourne une erreur
    assert_ne!(res.status(), StatusCode::OK);
}
