//! `with_cross_site_form_guard`: a form sent from another site without an
//! `X-CSRF-Token` header is refused before the handler, on the browser's
//! `Sec-Fetch-Site`. Off by default; when it is on, nothing sent from the site
//! itself, nor without the header, changes.

use crate::helpers::server::{build_default_router, build_engine};
use axum::{
    body::Body,
    http::{Method, Request, header},
    response::Response,
};
use runique::engine::RuniqueEngine;
use std::sync::Arc;
use tower::ServiceExt;

async fn engine(guard: bool, exempt: &[&str]) -> Arc<RuniqueEngine> {
    let mut engine = build_engine().await;
    let e = Arc::get_mut(&mut engine).expect("fresh engine");
    e.features.cross_site_form_guard = guard;
    e.csrf_exempt_paths = Arc::new(exempt.iter().map(|p| p.to_string()).collect());
    engine
}

/// A form `POST /submit`, with `Sec-Fetch-Site: site` when given.
async fn form_post(engine: Arc<RuniqueEngine>, uri: &str, site: Option<&str>) -> Response {
    let mut req = Request::builder()
        .method(Method::POST)
        .uri(uri)
        .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded");
    if let Some(site) = site {
        req = req.header("sec-fetch-site", site);
    }
    build_default_router(engine)
        .oneshot(req.body(Body::from("title=x")).unwrap())
        .await
        .unwrap()
}

#[tokio::test]
async fn off_by_default_a_form_from_another_site_reaches_the_token_check() {
    for site in ["cross-site", "same-site"] {
        let resp = form_post(engine(false, &[]).await, "/submit", Some(site)).await;
        assert_eq!(resp.status(), 200, "{site}: left to Prisme as before");
    }
}

#[tokio::test]
async fn on_a_form_from_another_domain_or_a_subdomain_is_refused() {
    for site in ["cross-site", "same-site", "Cross-Site", " same-site "] {
        let resp = form_post(engine(true, &[]).await, "/submit", Some(site)).await;
        assert_eq!(resp.status(), 403, "{site:?}");
    }
}

#[tokio::test]
async fn on_a_form_from_the_site_itself_or_without_the_header_goes_through() {
    for site in [Some("same-origin"), Some("none"), None, Some("other")] {
        let resp = form_post(engine(true, &[]).await, "/submit", site).await;
        assert_eq!(resp.status(), 200, "{site:?}");
    }
}

#[tokio::test]
async fn on_an_exempt_path_is_left_to_its_own_check() {
    let resp = form_post(
        engine(true, &["/submit"]).await,
        "/submit",
        Some("cross-site"),
    )
    .await;
    assert_eq!(resp.status(), 200);
}

#[tokio::test]
async fn on_a_get_from_another_site_is_not_checked() {
    let req = Request::builder()
        .method(Method::GET)
        .uri("/")
        .header("sec-fetch-site", "cross-site")
        .body(Body::empty())
        .unwrap();
    let resp = build_default_router(engine(true, &[]).await)
        .oneshot(req)
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
}

/// With an `X-CSRF-Token` header the header decides, as before: a wrong one
/// is refused for being wrong, not for the site it comes from.
#[tokio::test]
async fn on_a_request_with_the_csrf_header_is_decided_by_that_header() {
    let req = Request::builder()
        .method(Method::POST)
        .uri("/submit")
        .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
        .header("sec-fetch-site", "cross-site")
        .header("X-CSRF-Token", "not-a-token")
        .body(Body::from("title=x"))
        .unwrap();
    let resp = build_default_router(engine(true, &[]).await)
        .oneshot(req)
        .await
        .unwrap();
    assert_eq!(resp.status(), 403);
    let body = axum::body::to_bytes(resp.into_body(), 1024).await.unwrap();
    assert_eq!(&body[..], b"Invalid CSRF token");
}
