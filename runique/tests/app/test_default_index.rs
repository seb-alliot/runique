//! Runique's welcome page at `/`: served in debug only, and only when the app
//! surely leaves `/` free.
use axum::Router;
use axum::body::{Body, to_bytes};
use axum::http::{Request, StatusCode};
use axum::routing::get;
use runique::app::RuniqueApp;
use runique::config::RuniqueConfig;
use runique::macros::Routes;
use runique::urlpatterns;
use sea_orm::Database;
use serial_test::serial;
use tower::ServiceExt;

const WELCOME: &str = "Runique is running";

async fn build(debug: bool, routes: Option<Routes>) -> Router {
    let mut config = RuniqueConfig::from_env();
    config.debug = debug;
    config.server.secret_key = "a-long-enough-test-secret-key-for-production-0123456789".into();
    let builder = RuniqueApp::builder(config)
        .with_database(Database::connect("sqlite::memory:").await.unwrap());
    match routes {
        Some(routes) => builder.routes(routes).build().await,
        None => builder.build().await,
    }
    .expect("app builds")
    .router
}

async fn home(app: Router) -> (StatusCode, String) {
    let resp = app
        .oneshot(Request::builder().uri("/").body(Body::empty()).unwrap())
        .await
        .unwrap();
    let status = resp.status();
    let body = to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    (status, String::from_utf8_lossy(&body).into_owned())
}

fn without_root() -> Routes {
    urlpatterns! {
        "/about" => get(|| async { "about" }),
    }
}

#[tokio::test]
#[serial]
async fn debug_serves_the_welcome_page_when_root_is_undeclared() {
    let (status, body) = home(build(true, Some(without_root())).await).await;
    assert_eq!(status, StatusCode::OK);
    assert!(body.contains(WELCOME), "{body}");
    assert!(body.contains("https://runique.io/docs/en"), "{body}");
}

#[tokio::test]
#[serial]
async fn debug_serves_the_welcome_page_without_any_routes() {
    let (status, body) = home(build(true, None).await).await;
    assert_eq!(status, StatusCode::OK);
    assert!(body.contains(WELCOME), "{body}");
}

#[tokio::test]
#[serial]
async fn production_leaves_an_undeclared_root_as_404() {
    let (status, body) = home(build(false, Some(without_root())).await).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert!(!body.contains(WELCOME), "{body}");
}

#[tokio::test]
#[serial]
async fn a_declared_root_wins_over_the_welcome_page() {
    let routes = urlpatterns! {
        "/" => get(|| async { "home" }),
    };
    let (status, body) = home(build(true, Some(routes)).await).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body, "home");
}

#[tokio::test]
#[serial]
async fn a_plain_router_declaring_root_does_not_clash() {
    let router = Router::new().route("/", get(|| async { "home" }));
    let (status, body) = home(build(true, Some(router.into())).await).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body, "home");
}

#[tokio::test]
#[serial]
async fn a_merged_plain_router_turns_the_welcome_page_off() {
    // Its paths can't be read: `/` may be taken, so Runique stays out of it.
    let routes = without_root().merge(Router::new().route("/x", get(|| async { "x" })));
    let (status, body) = home(build(true, Some(routes)).await).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert!(!body.contains(WELCOME), "{body}");
}
