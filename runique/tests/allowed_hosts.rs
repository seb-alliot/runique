use axum::extract::Extension;
use axum::{
    body::Body,
    http::{header, Request, StatusCode},
    middleware,
    routing::get,
    Router,
};
use runique::{middleware::allowed_hosts::allowed_hosts_middleware, Settings};
use std::sync::Arc;
use tower::ServiceExt;

/// Handler de test simple
async fn test_handler() -> &'static str {
    "OK"
}

/// Crée une app de test avec le middleware allowed_hosts
fn create_test_app(allowed_hosts: Vec<String>, debug: bool) -> Router {
    let mut settings = Settings::default_values();
    settings.allowed_hosts = allowed_hosts;
    settings.debug = debug;

    Router::new()
        .route("/", get(test_handler))
        .layer(middleware::from_fn(allowed_hosts_middleware))
        .layer(Extension(Arc::new(settings)))
}

#[tokio::test]
async fn test_allowed_host_exact_match() {
    let app = create_test_app(vec!["example.com".to_string()], false);

    // Host autorisé
    let req = Request::builder()
        .uri("/")
        .header(header::HOST, "example.com")
        .body(Body::empty())
        .unwrap();

    let res = app.oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_allowed_host_rejected() {
    let app = create_test_app(vec!["example.com".to_string()], false);

    // Host non autorisé
    let req = Request::builder()
        .uri("/")
        .header(header::HOST, "malicious.com")
        .body(Body::empty())
        .unwrap();

    let res = app.oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_allowed_host_wildcard_subdomain() {
    let app = create_test_app(vec![".example.com".to_string()], false);

    // Sous-domaines autorisés
    let hosts = vec![
        "example.com",
        "www.example.com",
        "api.example.com",
        "admin.api.example.com",
    ];

    for host in hosts {
        let req = Request::builder()
            .uri("/")
            .header(header::HOST, host)
            .body(Body::empty())
            .unwrap();

        let res = app.clone().oneshot(req).await.unwrap();
        assert_eq!(
            res.status(),
            StatusCode::OK,
            "Host '{}' devrait être autorisé",
            host
        );
    }
}

#[tokio::test]
async fn test_allowed_host_wildcard_subdomain_security() {
    let app = create_test_app(vec![".example.com".to_string()], false);

    // Hosts malveillants qui ne doivent PAS être autorisés
    let malicious_hosts = vec![
        "malicious-example.com",
        "evil-example.com",
        "example.com.evil.com",
    ];

    for host in malicious_hosts {
        let req = Request::builder()
            .uri("/")
            .header(header::HOST, host)
            .body(Body::empty())
            .unwrap();

        let res = app.clone().oneshot(req).await.unwrap();
        assert_eq!(
            res.status(),
            StatusCode::BAD_REQUEST,
            "Host '{}' ne devrait PAS être autorisé (bug de sécurité)",
            host
        );
    }
}

#[tokio::test]
async fn test_allowed_host_wildcard_all() {
    let app = create_test_app(vec!["*".to_string()], false);

    // Tous les hosts devraient être autorisés
    let hosts = vec!["example.com", "any-domain.com", "malicious.com"];

    for host in hosts {
        let req = Request::builder()
            .uri("/")
            .header(header::HOST, host)
            .body(Body::empty())
            .unwrap();

        let res = app.clone().oneshot(req).await.unwrap();
        assert_eq!(
            res.status(),
            StatusCode::OK,
            "Host '{}' devrait être autorisé avec wildcard *",
            host
        );
    }
}

#[tokio::test]
async fn test_allowed_host_with_port() {
    let app = create_test_app(vec!["example.com".to_string()], false);

    // Host avec port devrait être autorisé (le port est retiré)
    let req = Request::builder()
        .uri("/")
        .header(header::HOST, "example.com:8080")
        .body(Body::empty())
        .unwrap();

    let res = app.oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_allowed_host_debug_mode() {
    let app = create_test_app(vec!["example.com".to_string()], true);

    // En mode debug, tous les hosts sont autorisés
    let req = Request::builder()
        .uri("/")
        .header(header::HOST, "any-host.com")
        .body(Body::empty())
        .unwrap();

    let res = app.oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_allowed_host_missing_header() {
    let app = create_test_app(vec!["example.com".to_string()], false);

    // Requête sans header Host
    let req = Request::builder().uri("/").body(Body::empty()).unwrap();

    let res = app.oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_allowed_host_multiple_allowed() {
    let app = create_test_app(
        vec![
            "example.com".to_string(),
            "test.com".to_string(),
            ".api.example.com".to_string(),
        ],
        false,
    );

    // Tous ces hosts devraient être autorisés
    let hosts = vec![
        "example.com",
        "test.com",
        "api.example.com",
        "v1.api.example.com",
    ];

    for host in hosts {
        let req = Request::builder()
            .uri("/")
            .header(header::HOST, host)
            .body(Body::empty())
            .unwrap();

        let res = app.clone().oneshot(req).await.unwrap();
        assert_eq!(
            res.status(),
            StatusCode::OK,
            "Host '{}' devrait être autorisé",
            host
        );
    }

    // Ce host ne devrait pas être autorisé
    let req = Request::builder()
        .uri("/")
        .header(header::HOST, "other.com")
        .body(Body::empty())
        .unwrap();

    let res = app.oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
}
