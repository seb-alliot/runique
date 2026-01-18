use axum::body::Body;
use axum::http::{Request, StatusCode};
use axum::routing::get;
use http_body_util::BodyExt;
use runique::prelude::*;

use tower::ServiceExt;

#[tokio::test]
async fn test_runique_app_async() {
    let settings = Settings::builder()
        .debug(true)
        .templates_dir(vec!["templates".to_string()])
        .server("127.0.0.1", 3000, "change_your_secret_key")
        .build();

    let runique_app = RuniqueApp::new(settings.clone())
        .await
        .expect("Failed to create RuniqueApp");

    // On build simplement. Le framework injecte l'AppState (Tera + Settings) tout seul.
    let app = runique_app
        .routes(axum::Router::new().route("/", get(|| async { "Hello World" })))
        .build();

    let request = Request::builder()
        .uri("/")
        .header("Host", "localhost")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = response.into_body().collect().await.unwrap().to_bytes();
    let body_str = String::from_utf8(body.to_vec()).unwrap();

    assert_eq!(body_str, "Hello World");
}

#[tokio::test]
async fn test_with_all_middleware() {
    let settings = Settings::builder()
        .debug(true)
        .allowed_hosts(vec!["localhost".to_string()])
        .server("127.0.0.1", 3001, "test_secret_key")
        .build();

    let runique_app = RuniqueApp::new(settings).await.unwrap();

    // build() s'occupe de transformer le Router<AppState> en Router<()> consommable
    let app = runique_app
        .routes(axum::Router::new().route("/", get(|| async { (StatusCode::OK, "Hello") })))
        .with_default_middleware()
        .build();

    let request = Request::builder()
        .uri("/")
        .header("Host", "localhost")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_rejected_host() {
    let settings = Settings::builder()
        .debug(false)
        .allowed_hosts(vec!["example.com".to_string()])
        .server("127.0.0.1", 3002, "secret")
        .build();

    let runique_app = RuniqueApp::new(settings).await.unwrap();

    let app = runique_app
        .routes(axum::Router::new().route("/", get(|| async { "Hello" })))
        .with_allowed_hosts(Some(vec!["example.com".to_string()]))
        .with_default_middleware()
        .build();

    let request = Request::builder()
        .uri("/")
        .header("Host", "evil.com")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}
