// Tests d'intégration pour RuniqueContext::from_request_parts
//
// Stratégie : router oneshot avec 3 couches :
//   session_layer (outermost) → engine_inject → csrf_middleware → handler
//
// Le handler extrait RuniqueContext — si l'extraction réussit → 200, sinon 500.

use crate::helpers::{
    request,
    server::{TEST_SECRET, build_engine},
};
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
    context::RequestExtensions, context::RuniqueContext,
    middleware::security::csrf::csrf_middleware, utils::aliases::AEngine,
};
use tower_sessions::{MemoryStore, SessionManagerLayer};

// ── Middleware injectant AEngine dans les extensions ────────────────────────

async fn engine_inject(
    State(engine): State<AEngine>,
    mut req: Request<Body>,
    next: Next,
) -> Response {
    RequestExtensions::new()
        .with_engine(engine)
        .inject_request(&mut req);
    next.run(req).await
}

// ── Handler qui extrait RuniqueContext ───────────────────────────────────────

async fn ctx_handler(ctx: RuniqueContext) -> impl IntoResponse {
    // Si on arrive ici, l'extraction a réussi
    let _ = ctx.engine.config.server.secret_key.len();
    StatusCode::OK
}

async fn ctx_method_handler(ctx: RuniqueContext) -> impl IntoResponse {
    if ctx.tpl.is_get() {
        StatusCode::OK
    } else {
        StatusCode::IM_A_TEAPOT
    }
}

// ── Builder du router de test ────────────────────────────────────────────────

async fn ctx_app() -> Router {
    let engine = build_engine().await;
    let session_layer = SessionManagerLayer::new(MemoryStore::default());

    Router::new()
        .route("/ctx", get(ctx_handler))
        .route("/ctx/method", get(ctx_method_handler))
        // Ordre des layers (dernier = outermost = premier à traiter la requête)
        .layer(middleware::from_fn_with_state(
            engine.clone(),
            csrf_middleware,
        ))
        .layer(middleware::from_fn_with_state(
            engine.clone(),
            engine_inject,
        ))
        .layer(session_layer)
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[tokio::test]
async fn test_runique_context_get_200() {
    let resp = request::get(ctx_app().await, "/ctx").await;
    assert_eq!(resp.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_runique_context_is_get_true() {
    let resp = request::get(ctx_app().await, "/ctx/method").await;
    assert_eq!(resp.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_runique_context_sans_engine_retourne_500() {
    // Router sans engine dans les extensions → extraction doit échouer (500)
    let session_layer = SessionManagerLayer::new(MemoryStore::default());
    let engine = build_engine().await;

    let app = Router::new()
        .route("/ctx", get(ctx_handler))
        .layer(middleware::from_fn_with_state(engine, csrf_middleware))
        // Pas de engine_inject → AEngine absent des extensions
        .layer(session_layer);

    let resp = request::get(app, "/ctx").await;
    assert_eq!(resp.status(), StatusCode::INTERNAL_SERVER_ERROR);
}

#[tokio::test]
async fn test_runique_context_secret_key_correcte() {
    // Vérifie que l'engine injecté est bien celui du test (secret connu)
    let engine = build_engine().await;
    assert_eq!(engine.config.server.secret_key, TEST_SECRET);
}
