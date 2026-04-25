//! Tests — macros/routeur/router_ext
//! Couvre : RouterExt::rate_limit, rate_limit_many, login_required (smoke tests)

use axum::{Router, routing::get};
use runique::macros::RouterExt;

#[tokio::test]
async fn test_rate_limit_builds_router() {
    let handler = get(|| async { "ok" });
    let _router: Router = Router::new().rate_limit("/test", "test_route", handler, 10, 60);
}

#[tokio::test]
async fn test_rate_limit_many_builds_router() {
    let handler1 = get(|| async { "h1" });
    let handler2 = get(|| async { "h2" });
    let _router: Router = Router::new().rate_limit_many(
        5,
        30,
        vec![
            ("/route-a".into(), "route_a".into(), handler1),
            ("/route-b".into(), "route_b".into(), handler2),
        ],
    );
}

#[tokio::test]
async fn test_login_required_builds_router() {
    let handler = get(|| async { "protected" });
    let _router: Router =
        Router::new().login_required("/dashboard", "dashboard", handler, "/login");
}
