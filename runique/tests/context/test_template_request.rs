// Tests pour context::template::Request — from_request_parts, render (erreur
// et succès), insert, map_tera.
//
// Stack : csrf_router — csrf_middleware (GET seulement, token validé).

use crate::helpers::{assert::body_str, request, server::build_engine};
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
    context::{RequestExtensions, template::Request as TplRequest},
    middleware::security::csrf::csrf_middleware,
    utils::aliases::{AEngine, AppResult},
};
use std::sync::Arc;
use tera::Tera;
use tower_sessions::{MemoryStore, SessionManagerLayer};

// ── Middlewares d'injection ─────────────────────────────────────────────────

/// Injecte engine + config (requis par prisme_pipeline dans Request::from_request).
async fn engine_inject(
    State(engine): State<AEngine>,
    mut req: Request<Body>,
    next: Next,
) -> Response {
    let config = Arc::new(engine.config.clone());
    RequestExtensions::new()
        .with_engine(engine)
        .with_config(config)
        .inject_request(&mut req);
    next.run(req).await
}

// ── Routers ─────────────────────────────────────────────────────────────────

/// Router GET seulement — utilise csrf_middleware (token réel).
fn csrf_router(engine: AEngine) -> Router {
    let session_layer = SessionManagerLayer::new(MemoryStore::default());
    Router::new()
        .route("/", get(handler_get_ok))
        .route("/render_err", get(handler_render_err))
        .route("/insert", get(handler_insert))
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

async fn default_csrf_app() -> Router {
    csrf_router(build_engine().await)
}

// ── Handlers ────────────────────────────────────────────────────────────────

async fn handler_get_ok(_tpl: TplRequest) -> impl IntoResponse {
    StatusCode::OK
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

/// render() succès — engine avec un vrai template.
async fn handler_render_ok(mut tpl: TplRequest) -> Response {
    tpl.render("hello.html")
        .unwrap_or_else(|e| e.into_response())
}

// ── Tests — from_request_parts ──────────────────────────────────────────────

#[tokio::test]
async fn test_request_extraction_get_200() {
    let resp = request::get(default_csrf_app().await, "/").await;
    assert_eq!(resp.status(), StatusCode::OK);
}

// ── Tests — render (chemin erreur) → AppError::map_tera ─────────────────────

#[tokio::test]
async fn test_render_template_not_found_returns_500() {
    let resp = request::get(default_csrf_app().await, "/render_err").await;
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
        csrf_exempt_paths: Arc::new(vec![]),
        permissions_policy: Arc::new(runique::middleware::PermissionsPolicy::default()),
        trusted_proxies: Arc::new(runique::middleware::TrustedProxies::default()),
        session_store: std::sync::LazyLock::new(|| std::sync::RwLock::new(None)),
        session_db_store: std::sync::LazyLock::new(|| std::sync::RwLock::new(None)),
        extensions: std::collections::HashMap::new(),
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
        .route("/", get(handler_get_ok))
        .layer(middleware::from_fn_with_state(
            engine.clone(),
            csrf_middleware,
        ))
        .layer(session_layer);

    let resp = request::get(app, "/").await;
    assert_eq!(resp.status(), StatusCode::INTERNAL_SERVER_ERROR);
}

// ── Path and query getters — written from cargo-mutants survivors (2026-10-02)

#[derive(serde::Deserialize, Default, Debug)]
struct ListQuery {
    page: Option<i32>,
    q: Option<String>,
}

async fn handler_params(tpl: TplRequest) -> AppResult<String> {
    let query: ListQuery = tpl.query()?;
    Ok(format!(
        "{:?}|{:?}|{:?}|{:?}|{:?}",
        tpl.get_path("id"),
        tpl.get_path("other"),
        tpl.get_query("page"),
        query.page,
        query.q
    ))
}

async fn params_app() -> Router {
    let engine = build_engine().await;
    Router::new()
        .route("/items/{id}", get(handler_params))
        .layer(middleware::from_fn_with_state(
            engine.clone(),
            csrf_middleware,
        ))
        .layer(middleware::from_fn_with_state(engine, engine_inject))
        .layer(SessionManagerLayer::new(MemoryStore::default()))
}

#[tokio::test]
async fn test_path_and_query_getters() {
    let resp = request::get(params_app().await, "/items/42?page=3&q=rust").await;
    assert_eq!(
        body_str(resp).await,
        r#"Some("42")|None|Some("3")|Some(3)|Some("rust")"#
    );
}

#[tokio::test]
async fn test_an_empty_query_value_does_not_reset_the_others() {
    // `page=` can't be read as an i32: dropped, instead of failing the whole struct.
    let resp = request::get(params_app().await, "/items/1?page=&q=rust").await;
    assert_eq!(
        body_str(resp).await,
        r#"Some("1")|None|Some("")|None|Some("rust")"#
    );
}

#[tokio::test]
async fn test_a_query_that_does_not_fit_is_a_bad_request() {
    let resp = request::get(params_app().await, "/items/1?page=abc&q=rust").await;
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

// ── Tests — public_url ──────────────────────────────────────────────────────

async fn handler_public_url(req: TplRequest) -> impl IntoResponse {
    req.public_url().unwrap_or_else(|| "none".to_string())
}

/// App whose engine has the given public URL and debug mode.
async fn public_url_app(public_url: Option<&str>, debug: bool) -> Router {
    use runique::engine::RuniqueEngine;
    use runique::middleware::{
        config::MiddlewareConfig,
        security::{allowed_hosts::HostPolicy, csp::SecurityPolicy},
    };
    let base = build_engine().await;
    let mut config = base.config.clone();
    config.server.public_url = public_url.map(str::to_string);
    config.debug = debug;
    let engine = Arc::new(RuniqueEngine {
        tera: base.tera.clone(),
        config,
        db: base.db.clone(),
        url_registry: base.url_registry.clone(),
        features: MiddlewareConfig::default(),
        security_csp: Arc::new(SecurityPolicy::default()),
        security_hosts: Arc::new(HostPolicy::new(vec![], true)),
        csrf_exempt_paths: Arc::new(vec![]),
        permissions_policy: Arc::new(runique::middleware::PermissionsPolicy::default()),
        trusted_proxies: Arc::new(runique::middleware::TrustedProxies::default()),
        session_store: std::sync::LazyLock::new(|| std::sync::RwLock::new(None)),
        session_db_store: std::sync::LazyLock::new(|| std::sync::RwLock::new(None)),
        extensions: std::collections::HashMap::new(),
    });
    Router::new()
        .route("/u", get(handler_public_url))
        .layer(middleware::from_fn_with_state(
            engine.clone(),
            csrf_middleware,
        ))
        .layer(middleware::from_fn_with_state(engine, engine_inject))
        .layer(SessionManagerLayer::new(MemoryStore::default()))
}

#[tokio::test]
async fn public_url_is_the_configured_one_whatever_the_host() {
    let app = public_url_app(Some("https://mysite.com"), false).await;
    let resp = request::get_with_header(app, "/u", "host", "evil.com").await;
    assert_eq!(body_str(resp).await, "https://mysite.com");
}

// A forged Host must never become the base of a link outside debug.
#[tokio::test]
async fn public_url_ignores_the_host_in_production() {
    let app = public_url_app(None, false).await;
    let resp = request::get_with_header(app, "/u", "host", "evil.com").await;
    assert_eq!(body_str(resp).await, "none");
}

#[tokio::test]
async fn public_url_falls_back_to_the_host_in_debug() {
    let app = public_url_app(None, true).await;
    let resp = request::get_with_header(app, "/u", "host", "localhost:3000").await;
    assert_eq!(body_str(resp).await, "http://localhost:3000");
}
