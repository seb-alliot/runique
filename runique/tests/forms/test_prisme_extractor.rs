// Tests d'intégration pour forms/extractor.rs — pipeline Aegis → CSRF
// intégré dans Request::from_request.
//
// Stratégie : router oneshot avec middleware injectant les extensions nécessaires.
// GET requests utilisées pour éviter la validation CSRF (check_csrf passe sur GET/HEAD).

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
use runique::{
    forms::{BodyRefusal, prisme_pipeline},
    http::RequestExtensions,
    security::csrf::CsrfToken,
    utils::aliases::AEngine,
};
use tower_sessions::{MemoryStore, SessionManagerLayer};

// ── Middleware : injecte l'engine (qui porte la config) dans les extensions ──

async fn full_inject(
    State(engine): State<AEngine>,
    mut req: Request<Body>,
    next: Next,
) -> Response {
    RequestExtensions::new()
        .with_engine(engine)
        .inject_request(&mut req);
    next.run(req).await
}

// ── Handler ──────────────────────────────────────────────────────────────────

async fn prisme_handler(request: runique::http::Request) -> impl IntoResponse {
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

// ── Tests ────────────────────────────────────────────────────────────────────

#[tokio::test]
async fn test_prisme_get_happy_path_200() {
    let resp = request::get(full_app().await, "/form").await;
    assert_status(&resp, 200);
}

// The config is read from the engine: without it, the body is refused
// before anything is read.
#[tokio::test]
async fn test_prisme_without_engine_is_refused_500() {
    let mut req = Request::builder().uri("/form").body(Body::empty()).unwrap();
    req.extensions_mut().insert(CsrfToken("t".repeat(64)));
    match prisme_pipeline(req, &()).await {
        Err(BodyRefusal::Status(status, _)) => {
            assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
        }
        other => panic!("expected a 500 refusal, got {:?}", other.map(|_| ())),
    }
}

// With the engine but no CSRF token in the extensions, the body is refused too.
#[tokio::test]
async fn test_prisme_with_engine_without_csrf_token_is_refused_500() {
    let mut req = Request::builder().uri("/form").body(Body::empty()).unwrap();
    req.extensions_mut().insert(build_engine().await);
    match prisme_pipeline(req, &()).await {
        Err(BodyRefusal::Status(status, _)) => {
            assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
        }
        other => panic!("expected a 500 refusal, got {:?}", other.map(|_| ())),
    }
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
