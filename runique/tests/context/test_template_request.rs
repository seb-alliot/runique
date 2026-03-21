// Tests pour context::template::Request — from_request_parts, is_get/post/put/delete,
// render (erreur et succès), insert, render_with, map_tera.
//
// Deux stacks :
//  • csrf_router  — csrf_middleware (GET seulement, token validé)
//  • method_router — inject bypass (POST/PUT/DELETE : token injecté sans validation)

use crate::helpers::{assert::body_str, request, server::build_engine};
use axum::{
    Router,
    body::Body,
    extract::State,
    http::{Method, Request, StatusCode},
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::{delete, get, post, put},
};
use runique::{
    context::{RequestExtensions, template::Request as TplRequest},
    middleware::security::csrf::csrf_middleware,
    utils::aliases::AEngine,
    utils::csrf::CsrfToken,
};
use std::sync::Arc;
use tera::Tera;
use tower::ServiceExt;
use tower_sessions::{MemoryStore, SessionManagerLayer};

// ── Middlewares d'injection ─────────────────────────────────────────────────

/// Injecte uniquement l'engine (utilisé avec csrf_middleware pour les GET).
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

/// Injecte l'engine + un CsrfToken factice — pour tester POST/PUT/DELETE
/// sans passer par la validation CSRF (on teste juste la méthode HTTP).
async fn full_bypass_inject(
    State(engine): State<AEngine>,
    mut req: Request<Body>,
    next: Next,
) -> Response {
    let dummy_token = CsrfToken("test_bypass_token".to_string());
    RequestExtensions::new()
        .with_engine(engine)
        .inject_request(&mut req);
    req.extensions_mut().insert(dummy_token);
    next.run(req).await
}

// ── Routers ─────────────────────────────────────────────────────────────────

/// Router GET seulement — utilise csrf_middleware (token réel).
fn csrf_router(engine: AEngine) -> Router {
    let session_layer = SessionManagerLayer::new(MemoryStore::default());
    Router::new()
        .route("/", get(handler_is_get))
        .route("/render_err", get(handler_render_err))
        .route("/insert", get(handler_insert))
        .route("/render_with_err", get(handler_render_with_err))
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

/// Router méthodes — bypass CSRF pour tester POST/PUT/DELETE.
fn method_router(engine: AEngine) -> Router {
    let session_layer = SessionManagerLayer::new(MemoryStore::default());
    Router::new()
        .route("/post", post(handler_is_post))
        .route("/put", put(handler_is_put))
        .route("/delete", delete(handler_is_delete))
        .layer(middleware::from_fn_with_state(
            engine.clone(),
            full_bypass_inject,
        ))
        .layer(session_layer)
}

async fn default_csrf_app() -> Router {
    csrf_router(build_engine().await)
}

async fn default_method_app() -> Router {
    method_router(build_engine().await)
}

// ── Handlers ────────────────────────────────────────────────────────────────

async fn handler_is_get(tpl: TplRequest) -> impl IntoResponse {
    if tpl.is_get() {
        StatusCode::OK
    } else {
        StatusCode::INTERNAL_SERVER_ERROR
    }
}

async fn handler_is_post(tpl: TplRequest) -> impl IntoResponse {
    if tpl.is_post() {
        StatusCode::OK
    } else {
        StatusCode::INTERNAL_SERVER_ERROR
    }
}

async fn handler_is_put(tpl: TplRequest) -> impl IntoResponse {
    if tpl.is_put() {
        StatusCode::OK
    } else {
        StatusCode::INTERNAL_SERVER_ERROR
    }
}

async fn handler_is_delete(tpl: TplRequest) -> impl IntoResponse {
    if tpl.is_delete() {
        StatusCode::OK
    } else {
        StatusCode::INTERNAL_SERVER_ERROR
    }
}

/// Render avec Tera vide → tera::Error → AppError::map_tera → 500.
async fn handler_render_err(mut tpl: TplRequest) -> Response {
    tpl.render("nonexistent.html")
        .unwrap_or_else(|e| e.into_response())
}

/// insert() ajoute une clé au contexte sans paniquer.
async fn handler_insert(tpl: TplRequest) -> impl IntoResponse {
    let _tpl = tpl.insert("test_key", serde_json::json!("test_value"));
    StatusCode::OK
}

/// render_with() avec Tera vide → erreur → 500.
async fn handler_render_with_err(tpl: TplRequest) -> Response {
    tpl.render_with(
        "nonexistent.html",
        vec![("extra", serde_json::json!("data"))],
    )
    .unwrap_or_else(|e| e.into_response())
}

/// render() succès — engine avec un vrai template.
async fn handler_render_ok(mut tpl: TplRequest) -> Response {
    tpl.render("hello.html")
        .unwrap_or_else(|e| e.into_response())
}

// ── Tests — from_request_parts + méthodes ───────────────────────────────────

#[tokio::test]
async fn test_request_extraction_get_200() {
    let resp = request::get(default_csrf_app().await, "/").await;
    assert_eq!(resp.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_request_is_get_true() {
    let resp = request::get(default_csrf_app().await, "/").await;
    assert_eq!(resp.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_request_is_post_true() {
    let resp = request::post(default_method_app().await, "/post").await;
    assert_eq!(resp.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_request_is_put_true() {
    let app = default_method_app().await;
    let req = Request::builder()
        .method(Method::PUT)
        .uri("/put")
        .body(Body::empty())
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_request_is_delete_true() {
    let resp = request::delete(default_method_app().await, "/delete").await;
    assert_eq!(resp.status(), StatusCode::OK);
}

// ── Tests — render (chemin erreur) → AppError::map_tera ─────────────────────

#[tokio::test]
async fn test_render_template_not_found_returns_500() {
    let resp = request::get(default_csrf_app().await, "/render_err").await;
    assert_eq!(resp.status(), StatusCode::INTERNAL_SERVER_ERROR);
}

#[tokio::test]
async fn test_render_with_template_not_found_returns_500() {
    let resp = request::get(default_csrf_app().await, "/render_with_err").await;
    assert_eq!(resp.status(), StatusCode::INTERNAL_SERVER_ERROR);
}

// ── Tests — insert ──────────────────────────────────────────────────────────

#[tokio::test]
async fn test_insert_does_not_panic() {
    let resp = request::get(default_csrf_app().await, "/insert").await;
    assert_eq!(resp.status(), StatusCode::OK);
}

// ── Tests — render succès (avec template Tera) ──────────────────────────────

async fn app_with_template() -> Router {
    let engine = build_engine().await;

    let mut tera = Tera::default();
    tera.add_raw_template("hello.html", "<h1>Hello</h1>")
        .unwrap();

    use runique::engine::RuniqueEngine;
    use runique::middleware::{
        config::MiddlewareConfig,
        security::{allowed_hosts::HostPolicy, csp::SecurityPolicy},
    };

    let engine_with_tpl = Arc::new(RuniqueEngine {
        tera: Arc::new(tera),
        config: engine.config.clone(),
        db: engine.db.clone(),
        url_registry: engine.url_registry.clone(),
        features: MiddlewareConfig::default(),
        security_csp: Arc::new(SecurityPolicy::default()),
        security_hosts: Arc::new(HostPolicy::new(vec![], true)),
        session_store: std::sync::OnceLock::new(),
    });

    let session_layer = SessionManagerLayer::new(MemoryStore::default());
    Router::new()
        .route("/render_ok", get(handler_render_ok))
        .layer(middleware::from_fn_with_state(
            engine_with_tpl.clone(),
            csrf_middleware,
        ))
        .layer(middleware::from_fn_with_state(
            engine_with_tpl.clone(),
            engine_inject,
        ))
        .layer(session_layer)
}

#[tokio::test]
async fn test_render_success_returns_200() {
    let app = app_with_template().await;
    let resp = request::get(app, "/render_ok").await;
    assert_eq!(resp.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_render_success_returns_html_content() {
    let app = app_with_template().await;
    let resp = request::get(app, "/render_ok").await;
    assert!(body_str(resp).await.contains("Hello"));
}

// ── Tests — extraction sans engine → 500 ─────────────────────────────────────

#[tokio::test]
async fn test_request_extraction_sans_engine_retourne_500() {
    let engine = build_engine().await;
    let session_layer = SessionManagerLayer::new(MemoryStore::default());

    // Pas de engine_inject → AEngine absent → extraction échoue → 500
    let app = Router::new()
        .route("/", get(handler_is_get))
        .layer(middleware::from_fn_with_state(
            engine.clone(),
            csrf_middleware,
        ))
        .layer(session_layer);

    let resp = request::get(app, "/").await;
    assert_eq!(resp.status(), StatusCode::INTERNAL_SERVER_ERROR);
}
