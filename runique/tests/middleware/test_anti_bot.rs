//! The honeypot middleware gives each session one random field name and keeps
//! it for the session's next requests. Written from cargo-mutants survivors
//! (2026-10-08): anti_bot.rs:25.
use crate::helpers::server::build_engine;
use axum::{Extension, Router, body::Body, http::Request, middleware, routing::get};
use http_body_util::BodyExt;
use runique::middleware::security::{HoneypotFieldName, anti_bot_middleware};
use tower::ServiceExt;
use tower_sessions::{MemoryStore, SessionManagerLayer};

async fn app() -> Router {
    let engine = build_engine().await;
    Router::new()
        .route(
            "/",
            get(|Extension(name): Extension<HoneypotFieldName>| async move { name.0 }),
        )
        .layer(middleware::from_fn_with_state(engine, anti_bot_middleware))
        .layer(SessionManagerLayer::new(MemoryStore::default()).with_secure(false))
}

async fn fetch(app: Router, cookie: Option<&str>) -> (String, Option<String>) {
    let mut req = Request::builder().uri("/");
    if let Some(c) = cookie {
        req = req.header("cookie", c);
    }
    let resp = app.oneshot(req.body(Body::empty()).unwrap()).await.unwrap();
    let cookie = resp
        .headers()
        .get("set-cookie")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.split(';').next())
        .map(str::to_string);
    let body = resp.into_body().collect().await.unwrap().to_bytes();
    (String::from_utf8_lossy(&body).into_owned(), cookie)
}

#[tokio::test]
async fn each_session_keeps_its_own_random_field_name() {
    let app = app().await;
    let (first, cookie) = fetch(app.clone(), None).await;
    assert_eq!(first.len(), 16, "{first:?}");
    assert!(first.chars().all(|c| c.is_ascii_hexdigit()), "{first:?}");

    let cookie = cookie.expect("a session cookie");
    let (again, _) = fetch(app.clone(), Some(&cookie)).await;
    assert_eq!(again, first, "the same session keeps its name");

    let (other, _) = fetch(app, None).await;
    assert_ne!(other, first, "another session gets another name");
}
