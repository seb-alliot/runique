//! Tests — macros/routeur/router_ext
//! Couvre : RouterExt::rate_limit, rate_limit_many, login_required (smoke tests)

use axum::{Router, routing::get};
use runique::macros::RouterExt;

use crate::helpers::request;

#[tokio::test]
async fn test_rate_limit_builds_router() {
    let handler = get(|| async { "ok" });
    let _router: Router = Router::new().rate_limit("/test", "test_route", handler, 10, 60, vec![]);
}

#[tokio::test]
async fn test_rate_limit_many_builds_router() {
    let handler1 = get(|| async { "h1" });
    let handler2 = get(|| async { "h2" });
    let _router: Router = Router::new().rate_limit_many(
        5,
        30,
        vec![],
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

// Written from cargo-mutants survivors (2026-10-07): the tests above only
// built the router, so a `rate_limit` that dropped the route or the limit passed.
#[tokio::test]
async fn test_rate_limit_serves_the_route_then_limits_it() {
    let app: Router = Router::new().rate_limit(
        "/limited",
        "limited_route",
        get(|| async { "ok" }),
        1,
        60,
        vec![],
    );
    assert_eq!(request::get(app.clone(), "/limited").await.status(), 200);
    assert_eq!(request::get(app, "/limited").await.status(), 429);
}

#[tokio::test]
async fn test_rate_limit_many_limits_each_route() {
    let app: Router = Router::new().rate_limit_many(
        1,
        60,
        vec![],
        vec![("/many-a".into(), "many_a".into(), get(|| async { "a" }))],
    );
    assert_eq!(request::get(app.clone(), "/many-a").await.status(), 200);
    assert_eq!(request::get(app, "/many-a").await.status(), 429);
}

/// A limit restricted to some methods counts only those: other methods pass
/// freely. Written from cargo-mutants survivors (2026-10-08): rate_limit.rs:187.
#[tokio::test]
async fn test_rate_limit_counts_only_its_methods() {
    let app: Router = Router::new().rate_limit(
        "/form",
        "form_route",
        get(|| async { "page" }).post(|| async { "sent" }),
        1,
        60,
        vec![axum::http::Method::POST],
    );
    for _ in 0..3 {
        assert_eq!(
            request::get(app.clone(), "/form").await.status(),
            200,
            "GET is not limited"
        );
    }
    assert_eq!(request::post(app.clone(), "/form").await.status(), 200);
    assert_eq!(request::post(app, "/form").await.status(), 429, "POST is");
}
