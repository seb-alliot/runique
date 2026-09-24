use crate::helpers::{assert::assert_status, server::build_engine};
use axum::{
    Router,
    body::Body,
    http::{Request, StatusCode},
    middleware,
    routing::{get, post},
};
use runique::{
    engine::RuniqueEngine,
    middleware::{
        config::MiddlewareConfig,
        security::{allowed_hosts::HostPolicy, csp::SecurityPolicy, csrf::csrf_middleware},
    },
    utils::crypto::csrf::CsrfToken,
};
use std::sync::Arc;
use tower::ServiceExt;
use tower_sessions::{MemoryStore, SessionManagerLayer};

// ── Helpers ───────────────────────────────────────────────────────────────────

async fn engine_with_exempt(paths: Vec<&str>) -> Arc<RuniqueEngine> {
    let base = build_engine().await;
    Arc::new(RuniqueEngine {
        config: base.config.clone(),
        tera: base.tera.clone(),
        db: base.db.clone(),
        url_registry: base.url_registry.clone(),
        features: MiddlewareConfig::default(),
        security_csp: Arc::new(SecurityPolicy::default()),
        security_hosts: Arc::new(HostPolicy::new(vec![], true)),
        csrf_exempt_paths: Arc::new(paths.iter().map(|s| s.to_string()).collect()),
        permissions_policy: Arc::new(runique::middleware::PermissionsPolicy::default()),
        trusted_proxies: Arc::new(runique::middleware::TrustedProxies::default()),
        session_store: std::sync::LazyLock::new(|| std::sync::RwLock::new(None)),
        session_db_store: std::sync::LazyLock::new(|| std::sync::RwLock::new(None)),
        extensions: std::collections::HashMap::new(),
    })
}

/// Returns 200 if a `CsrfToken` extension is present on the request, 404 otherwise —
/// used to check that exempting a path from CSRF *validation* doesn't also skip
/// *generating* the token (which would break `Request`/`RuniqueContext` for any
/// exempt handler that still wants them).
async fn check_csrf_extension(req: Request<Body>) -> StatusCode {
    if req.extensions().get::<CsrfToken>().is_some() {
        StatusCode::OK
    } else {
        StatusCode::NOT_FOUND
    }
}

fn csrf_app(engine: Arc<RuniqueEngine>) -> Router {
    Router::new()
        .route("/", get(|| async { "ok" }))
        .route("/submit", post(|| async { "ok" }))
        .route("/webhook/stripe", post(|| async { "ok" }))
        .route("/webhook/github", post(|| async { "ok" }))
        .route("/webhook/stripe/sub", post(|| async { "ok" }))
        .route("/webhook/check-token", post(check_csrf_extension))
        .layer(middleware::from_fn_with_state(engine, csrf_middleware))
        .layer(SessionManagerLayer::new(MemoryStore::default()))
}

fn json_post(uri: &str) -> Request<Body> {
    Request::builder()
        .method("POST")
        .uri(uri)
        .header("Content-Type", "application/json")
        .body(Body::from(r#"{"event":"test"}"#))
        .unwrap()
}

// ── Comportement de base (sans exemption) ─────────────────────────────────────

#[tokio::test]
async fn json_post_sans_exempt_bloque() {
    let engine = build_engine().await;
    let app = csrf_app(engine);
    let resp = app.oneshot(json_post("/submit")).await.unwrap();
    assert_status(&resp, 403);
}

// ── Chemin exempté ────────────────────────────────────────────────────────────

#[tokio::test]
async fn json_post_sur_chemin_exempte_passe() {
    let engine = engine_with_exempt(vec!["/webhook/stripe"]).await;
    let app = csrf_app(engine);
    let resp = app.oneshot(json_post("/webhook/stripe")).await.unwrap();
    assert_status(&resp, 200);
}

#[tokio::test]
async fn plusieurs_chemins_exempts() {
    let engine = engine_with_exempt(vec!["/webhook/stripe", "/webhook/github"]).await;
    let app = csrf_app(engine);

    let resp_stripe = app
        .clone()
        .oneshot(json_post("/webhook/stripe"))
        .await
        .unwrap();
    assert_status(&resp_stripe, 200);

    let resp_github = app.oneshot(json_post("/webhook/github")).await.unwrap();
    assert_status(&resp_github, 200);
}

// ── Chemin non exempté reste bloqué ──────────────────────────────────────────

#[tokio::test]
async fn chemin_non_exempte_reste_bloque() {
    let engine = engine_with_exempt(vec!["/webhook/stripe"]).await;
    let app = csrf_app(engine);
    // /submit n'est pas exempté
    let resp = app.oneshot(json_post("/submit")).await.unwrap();
    assert_status(&resp, 403);
}

// ── Correspondance exacte (pas de sous-chemin) ───────────────────────────────

#[tokio::test]
async fn exemption_ne_couvre_pas_les_sous_chemins() {
    // /webhook/stripe ne doit pas exempter /webhook/stripe/sub
    let engine = engine_with_exempt(vec!["/webhook/stripe"]).await;
    let app = csrf_app(engine);
    let resp = app.oneshot(json_post("/webhook/stripe/sub")).await.unwrap();
    assert_status(&resp, 403);
}

// ── GET reste fonctionnel sur les chemins exemptés ───────────────────────────

#[tokio::test]
async fn get_sur_chemin_exempte_retourne_200() {
    let engine = engine_with_exempt(vec!["/webhook/stripe"]).await;
    let app = csrf_app(engine);
    let req = Request::builder()
        .method("GET")
        .uri("/")
        .body(Body::empty())
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_status(&resp, 200);
}

// ── Aucune exemption configurée ───────────────────────────────────────────────

#[tokio::test]
async fn liste_vide_bloque_tout() {
    let engine = engine_with_exempt(vec![]).await;
    let app = csrf_app(engine);
    let resp = app.oneshot(json_post("/webhook/stripe")).await.unwrap();
    assert_status(&resp, 403);
}

// ── Le token reste généré/injecté même sur un chemin exempté ────────────────────
// Régression : `csrf_exempt()` ne doit sauter que la *validation*, pas la
// génération du token — sinon `Request`/`RuniqueContext` plantent (500) sur tout
// handler exempté qui les utilise pour autre chose que le CSRF.

#[tokio::test]
async fn chemin_exempte_a_quand_meme_le_token_en_extension() {
    let engine = engine_with_exempt(vec!["/webhook/check-token"]).await;
    let app = csrf_app(engine);
    let resp = app
        .oneshot(json_post("/webhook/check-token"))
        .await
        .unwrap();
    assert_status(&resp, 200);
}
