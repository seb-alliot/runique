//! Static and media files served by a built app, with their headers — written
//! from cargo-mutants survivors (2026-10-02): `attach_static_files` and the
//! cache builders could be dropped without a test failing.
use axum::Router;
use axum::body::Body;
use axum::http::{Request, StatusCode};
use axum::routing::get;
use runique::app::RuniqueApp;
use runique::config::RuniqueConfig;
use sea_orm::Database;
use serial_test::serial;
use tower::ServiceExt;

async fn build(
    statics: impl FnOnce(runique::app::staging::StaticStaging) -> runique::app::staging::StaticStaging,
) -> (Result<Router, String>, std::path::PathBuf) {
    let root = std::env::temp_dir().join(format!("rq_static_{}", uuid::Uuid::new_v4()));
    let (static_dir, media_dir) = (root.join("static"), root.join("media"));
    std::fs::create_dir_all(&static_dir).unwrap();
    std::fs::create_dir_all(&media_dir).unwrap();
    std::fs::write(static_dir.join("app.css"), "body{}").unwrap();
    std::fs::write(media_dir.join("photo.txt"), "x").unwrap();

    let mut config = RuniqueConfig::from_env();
    config.debug = true;
    config.static_files.staticfiles_dirs = static_dir.to_string_lossy().into_owned();
    config.static_files.media_root = media_dir.to_string_lossy().into_owned();
    config.static_files.static_url = "/static".into();
    config.static_files.media_url = "/media".into();

    let app = RuniqueApp::builder(config)
        .with_database(Database::connect("sqlite::memory:").await.unwrap())
        .routes(Router::new().route("/", get(|| async { "ok" })))
        .static_files(statics)
        .build()
        .await
        .map(|a| a.router)
        .map_err(|e| e.to_string());
    (app, root)
}

async fn fetch(app: Router, uri: &str) -> axum::response::Response {
    app.oneshot(Request::builder().uri(uri).body(Body::empty()).unwrap())
        .await
        .unwrap()
}

fn header<'a>(resp: &'a axum::response::Response, name: &str) -> &'a str {
    resp.headers().get(name).map_or("", |v| v.to_str().unwrap())
}

#[tokio::test]
#[serial]
async fn static_and_media_files_carry_their_cache_and_security_headers() {
    let (app, root) = build(|s| s.static_cache("public, max-age=60").media_cache("no-cache")).await;
    let app = app.unwrap();

    let css = fetch(app.clone(), "/static/app.css").await;
    assert_eq!(css.status(), StatusCode::OK);
    assert_eq!(header(&css, "cache-control"), "public, max-age=60");
    assert_eq!(header(&css, "x-content-type-options"), "nosniff");
    assert_eq!(header(&css, "x-frame-options"), "DENY");

    let media = fetch(app, "/media/photo.txt").await;
    assert_eq!(media.status(), StatusCode::OK);
    assert_eq!(header(&media, "cache-control"), "no-cache");
    assert_eq!(header(&media, "x-content-type-options"), "nosniff");
    let _ = std::fs::remove_dir_all(root);
}

#[tokio::test]
#[serial]
async fn an_invalid_cache_header_is_a_build_error_not_a_panic() {
    let (app, root) = build(|s| s.static_cache("public\nmax-age=60")).await;
    let err = app.expect_err("refused");
    assert!(err.contains("static_cache"), "{err}");
    let (app, root2) = build(|s| s.enabled(false).static_cache("bad\nvalue")).await;
    assert!(app.is_ok(), "not checked when static files are off");
    let _ = std::fs::remove_dir_all(root);
    let _ = std::fs::remove_dir_all(root2);
}
