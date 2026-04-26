// Tests d'intégration pour forms/extractor.rs — pipeline Sentinel → CSRF → Aegis
// intégré dans Request::from_request.
//
// Stratégie : router oneshot avec middleware injectant les extensions nécessaires.
// GET requests utilisées pour éviter la validation CSRF (csrf_gate passe sur GET).

use crate::helpers::{assert::assert_status, request, server::build_engine};
use axum::{
    Router,
    body::Body,
    extract::State,
    http::{Request, StatusCode},
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::get,
};
use runique::{context::RequestExtensions, utils::aliases::AEngine};
use std::sync::Arc;
use tower_sessions::{MemoryStore, SessionManagerLayer};

// ── Middleware : injecte engine + config dans les extensions ─────────────────

async fn full_inject(
    State(engine): State<AEngine>,
    mut req: Request<Body>,
    next: Next,
) -> Response {
    let config = Arc::new(engine.config.clone());
    RequestExtensions::new()
        .with_engine(engine.clone())
        .with_tera(engine.tera.clone())
        .with_config(config)
        .inject_request(&mut req);
    next.run(req).await
}

async fn no_config_inject(
    State(engine): State<AEngine>,
    mut req: Request<Body>,
    next: Next,
) -> Response {
    // Omet ARuniqueConfig intentionnellement — prisme_pipeline échoue
    RequestExtensions::new()
        .with_engine(engine.clone())
        .with_tera(engine.tera.clone())
        .inject_request(&mut req);
    next.run(req).await
}

// ── Handler ──────────────────────────────────────────────────────────────────

async fn prisme_handler(request: runique::context::Request) -> impl IntoResponse {
    let _ = request.prisme.csrf_valid;
    StatusCode::OK
}

// ── Builders de router ───────────────────────────────────────────────────────

async fn full_app() -> Router {
    let engine = build_engine().await;
    let session_layer = SessionManagerLayer::new(MemoryStore::default());
    Router::new()
        .route("/form", get(prisme_handler))
        .layer(middleware::from_fn_with_state(
            engine.clone(),
            runique::middleware::security::csrf::csrf_middleware,
        ))
        .layer(middleware::from_fn_with_state(engine.clone(), full_inject))
        .layer(session_layer)
}

async fn no_config_app() -> Router {
    let engine = build_engine().await;
    let session_layer = SessionManagerLayer::new(MemoryStore::default());
    Router::new()
        .route("/form", get(prisme_handler))
        .layer(middleware::from_fn_with_state(
            engine.clone(),
            runique::middleware::security::csrf::csrf_middleware,
        ))
        .layer(middleware::from_fn_with_state(
            engine.clone(),
            no_config_inject,
        ))
        .layer(session_layer)
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[tokio::test]
async fn test_prisme_get_happy_path_200() {
    let resp = request::get(full_app().await, "/form").await;
    assert_status(&resp, 200);
}

#[tokio::test]
async fn test_prisme_get_missing_config_500() {
    let resp = request::get(no_config_app().await, "/form").await;
    assert_status(&resp, 500);
}

#[tokio::test]
async fn test_prisme_get_missing_csrf_token_500() {
    // Pas de csrf_middleware → CsrfToken absent des extensions → Request::from_request échoue
    let engine = build_engine().await;
    let session_layer = SessionManagerLayer::new(MemoryStore::default());
    let app = Router::new()
        .route("/form", get(prisme_handler))
        .layer(middleware::from_fn_with_state(engine.clone(), full_inject))
        .layer(session_layer);

    let resp = request::get(app, "/form").await;
    assert_status(&resp, 500);
}

#[tokio::test]
async fn test_prisme_get_with_query_params() {
    // GET avec query params → aegis les lit depuis l'URL → 200
    let resp = request::get(full_app().await, "/form?nom=alice&age=30").await;
    assert_status(&resp, 200);
}
