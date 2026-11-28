// Tests pour http::request::Request — from_request_parts, render (erreur
// et succès), insert, map_tera.
//
// Stack : csrf_router — csrf_middleware (GET seulement, token validé).

use crate::helpers::{
    assert::body_str,
    request,
    server::{build_engine, build_engine_with},
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
    http::{RequestExtensions, request::Request as TplRequest},
    middleware::security::csrf::csrf_middleware,
    utils::aliases::{AEngine, AppResult},
};
use std::sync::Arc;
use tera::Tera;
use tower_sessions::{MemoryStore, SessionManagerLayer};

// ── Middlewares d'injection ─────────────────────────────────────────────────

/// Injecte l'engine, qui porte la config lue par prisme_pipeline dans Request::from_request.
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
    let mut tera = Tera::default();
    tera.add_raw_template("hello.html", "<h1>Hello</h1>")
        .unwrap();

    let engine_with_tpl = build_engine_with(|e| e.tera = Arc::new(tera)).await;

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

async fn handler_typed_path(tpl: TplRequest) -> String {
    format!(
        "{:?}|{:?}",
        tpl.get_path_as::<i32>("id"),
        tpl.get_path_as::<i32>("other")
    )
}

async fn params_app() -> Router {
    let engine = build_engine().await;
    Router::new()
        .route("/items/{id}", get(handler_params))
        .route("/typed/{id}", get(handler_typed_path))
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

// Written from cargo-mutants survivors (2026-10-08): `get_path_as` was never called.
#[tokio::test]
async fn test_get_path_as_parses_or_gives_none() {
    let resp = request::get(params_app().await, "/typed/42").await;
    assert_eq!(body_str(resp).await, "Some(42)|None");
    let resp = request::get(params_app().await, "/typed/abc").await;
    assert_eq!(body_str(resp).await, "None|None");
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

/// App whose engine has the given public URL, in development or production.
async fn public_url_app(public_url: Option<&str>, dev: bool) -> Router {
    let engine = build_engine_with(|e| {
        let config = Arc::make_mut(&mut e.config);
        config.server.public_url = public_url.map(str::to_string);
        config.dev = dev;
    })
    .await;
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

// A forged Host must never become the base of a link, in either mode.
#[tokio::test]
async fn public_url_never_uses_the_host() {
    for dev in [false, true] {
        let app = public_url_app(None, dev).await;
        let resp = request::get_with_header(app, "/u", "host", "evil.com").await;
        assert_eq!(body_str(resp).await, "none", "dev: {dev}");
    }
}

// ── get_or_404 : l'erreur du handler, rendue avec `?` ───────────────────────

use runique::{
    auth::user::Entity as UserEntity, errors::ErrorContext, schema::Pk, sea_orm::ConnectionTrait,
};

/// By the primary key. `?table` creates the accounts table first: without
/// it the lookup itself fails.
async fn handler_get_or_404(tpl: TplRequest) -> AppResult<Response> {
    if tpl.query.contains_key("table") {
        let create = runique::migrations_table::create_eihwaz_users_table();
        tpl.engine.db.execute(&create).await.expect("create table");
    }
    let user = UserEntity::objects
        .get_or_404(&tpl.engine.db, Pk::default(), &tpl, "Compte introuvable")
        .await?;
    Ok(user.username.into_response())
}

/// The same through a query.
async fn handler_query_get_or_404(tpl: TplRequest) -> AppResult<Response> {
    if tpl.query.contains_key("table") {
        let create = runique::migrations_table::create_eihwaz_users_table();
        tpl.engine.db.execute(&create).await.expect("create table");
    }
    let user = UserEntity::objects
        .all()
        .get_or_404(&tpl.engine.db, &tpl, "Compte introuvable")
        .await?;
    Ok(user.username.into_response())
}

async fn get_or_404_app() -> Router {
    let engine = build_engine().await;
    let session_layer = SessionManagerLayer::new(MemoryStore::default());
    Router::new()
        .route("/by_key", get(handler_get_or_404))
        .route("/by_query", get(handler_query_get_or_404))
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

// No row: a 404 carrying the context the error middleware renders, with the
// message the handler gave.
#[tokio::test]
async fn get_or_404_without_the_row_is_a_404_with_its_message() {
    for path in ["/by_key?table=1", "/by_query?table=1"] {
        let resp = request::get(get_or_404_app().await, path).await;

        assert_eq!(resp.status(), StatusCode::NOT_FOUND, "{path}");
        let context = resp
            .extensions()
            .get::<Arc<ErrorContext>>()
            .unwrap_or_else(|| panic!("{path}: no context"));
        assert_eq!(context.status_code, 404, "{path}");
        assert_eq!(context.message, "Compte introuvable", "{path}");
    }
}

// The lookup fails (no table): a 500 with its own context, not a 404.
#[tokio::test]
async fn get_or_404_on_a_failed_lookup_is_a_500() {
    for path in ["/by_key", "/by_query"] {
        let resp = request::get(get_or_404_app().await, path).await;

        assert_eq!(resp.status(), StatusCode::INTERNAL_SERVER_ERROR, "{path}");
        let context = resp
            .extensions()
            .get::<Arc<ErrorContext>>()
            .unwrap_or_else(|| panic!("{path}: no context"));
        assert_eq!(context.status_code, 500, "{path}");
        assert_ne!(context.message, "Compte introuvable", "{path}");
    }
}
