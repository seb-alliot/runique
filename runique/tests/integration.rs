use axum::body::Body;
use axum::http::{Request, StatusCode};
use axum::routing::get;
use axum::Extension;
use http_body_util::BodyExt;
use runique::prelude::*;
use std::sync::Arc;
use tera::Tera;
use tower::ServiceExt;
use tower_sessions::{MemoryStore, SessionManagerLayer};

#[tokio::test]
async fn test_runique_app_async() {
    let settings = Settings::builder()
        .debug(true)
        .templates_dir(vec!["templates".to_string()])
        .server("127.0.0.1", 3000, "change_your_secret_key")
        .build();

    let settings_arc = Arc::new(settings.clone());
    let tera_arc = Arc::new(Tera::default());

    let runique_app = RuniqueApp::new(settings.clone())
        .await
        .expect("Failed to create RuniqueApp");

    let app = runique_app
        .routes(axum::Router::new().route("/", get(|| async { "Hello World" })))
        .build()
        .layer(Extension(settings_arc))
        .layer(Extension(tera_arc.clone()))
        .with_state(tera_arc); // <--- On passe l'Arc<Tera> au lieu de ()

    let request = Request::builder()
        .uri("/")
        .header("Host", "localhost")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    let status = response.status();
    let body = response.into_body().collect().await.unwrap().to_bytes();
    let body_str = String::from_utf8(body.to_vec()).unwrap();

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body_str, "Hello World");
}

#[tokio::test]
async fn test_with_all_middleware() {
    let settings = Settings::builder()
        .debug(true)
        .templates_dir(vec!["templates".to_string()])
        .allowed_hosts(vec!["localhost".to_string()])
        .server("127.0.0.1", 3000, "test_secret_key")
        .build();

    let settings_arc = Arc::new(settings.clone());
    let tera_arc = Arc::new(Tera::default());

    let runique_app = RuniqueApp::new(settings.clone()).await.unwrap();

    let session_store = MemoryStore::default();
    let session_layer = SessionManagerLayer::new(session_store).with_secure(false);

    let app = runique_app
        .routes(axum::Router::new().route("/", get(|| async { (StatusCode::OK, "Hello") })))
        .with_default_middleware()
        .build()
        .layer(session_layer)
        .layer(Extension(settings_arc))
        .layer(Extension(tera_arc.clone()))
        .with_state(tera_arc); // <--- On passe l'Arc<Tera> ici aussi

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
        .server("127.0.0.1", 3000, "secret")
        .build();

    let settings_arc = Arc::new(settings.clone());
    let tera_arc = Arc::new(Tera::default());

    let runique_app = RuniqueApp::new(settings.clone()).await.unwrap();

    let session_store = MemoryStore::default();
    let session_layer = SessionManagerLayer::new(session_store).with_secure(false);

    let app = runique_app
        .routes(axum::Router::new().route("/", get(|| async { "Hello" })))
        .with_allowed_hosts(Some(vec!["example.com".to_string()]))
        .with_default_middleware()
        .build()
        .layer(session_layer)
        .layer(Extension(settings_arc))
        .layer(Extension(tera_arc.clone()))
        .with_state(tera_arc); // <--- Et ici

    let request = Request::builder()
        .uri("/")
        .header("Host", "evil.com")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}
