//! Tests — `Routes`, the set `urlpatterns!` returns: its own methods and the
//! `RouterExt` ones keep every route they are given.

use axum::http::{Request, StatusCode, header};
use axum::routing::get;
use runique::app::RuniqueApp;
use runique::config::RuniqueConfig;
use runique::macros::{RouterExt, Routes};
use sea_orm::Database;
use serial_test::serial;
use tower::ServiceExt;

async fn get_status(app: &axum::Router, uri: &str) -> (StatusCode, Option<String>) {
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(uri)
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let location = resp
        .headers()
        .get(header::LOCATION)
        .and_then(|v| v.to_str().ok())
        .map(str::to_owned);
    (resp.status(), location)
}

#[tokio::test]
#[serial]
async fn every_route_added_to_a_routes_set_is_served() {
    let routes = Routes::new()
        .route("/plain", get(|| async { "plain" }))
        .login_required(
            "/private",
            "routes_ext_private",
            get(|| async { "private" }),
            "/sign-in",
        )
        .rate_limit(
            "/limited",
            "routes_ext_limited",
            get(|| async { "limited" }),
            5,
            60,
            vec![],
        )
        .rate_limit_many(
            5,
            60,
            vec![],
            vec![(
                "/shared".into(),
                "routes_ext_shared".into(),
                get(|| async { "shared" }),
            )],
        );

    let mut config = RuniqueConfig::from_env();
    config.debug = true;
    let app = RuniqueApp::builder(config)
        .with_database(Database::connect("sqlite::memory:").await.unwrap())
        .routes(routes)
        .static_files(|s| s.enabled(false))
        .build()
        .await
        .unwrap()
        .router;

    assert_eq!(get_status(&app, "/plain").await.0, StatusCode::OK);
    assert_eq!(
        get_status(&app, "/private").await,
        (StatusCode::SEE_OTHER, Some("/sign-in".to_string()))
    );
    assert_eq!(get_status(&app, "/limited").await.0, StatusCode::OK);
    assert_eq!(get_status(&app, "/shared").await.0, StatusCode::OK);
}
