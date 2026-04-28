// Tests pour RuniqueApp — builder pattern + validation de la config

use axum::{Router, routing::get};
use runique::{
    app::{RuniqueApp, RuniqueAppBuilder},
    config::RuniqueConfig,
};
use sea_orm::Database;

// ── RuniqueApp::builder ───────────────────────────────────────────

#[test]
fn test_builder_returns_builder() {
    let config = RuniqueConfig::default();
    // builder() ne doit pas paniquer et retourner un RuniqueAppBuilder
    let _builder: RuniqueAppBuilder = RuniqueApp::builder(config);
}

#[test]
fn test_builder_config_preserved() {
    let mut config = RuniqueConfig::default();
    config.server.secret_key = "ma_cle_secrete".to_string();
    // On vérifie juste que builder() accepte n'importe quelle config
    let _builder = RuniqueApp::builder(config);
}

// ── RuniqueAppBuilder — méthodes builder ─────────────────────────

#[test]
fn test_builder_routes_accepte_router() {
    let config = RuniqueConfig::default();
    let router = Router::new().route("/", get(|| async { "ok" }));
    let _builder = RuniqueApp::builder(config).routes(router);
}

#[test]
fn test_builder_middleware_closure() {
    let config = RuniqueConfig::default();
    let _builder = RuniqueApp::builder(config).middleware(|m| m);
}

#[test]
fn test_builder_core_closure() {
    let config = RuniqueConfig::default();
    let _builder = RuniqueApp::builder(config).core(|c| c);
}

// ── build() avec base de données en mémoire ──────────────────────

#[tokio::test]
async fn test_builder_build_avec_sqlite_memory() {
    let db = Database::connect("sqlite::memory:").await.unwrap();
    let mut config = RuniqueConfig::from_env();
    config.debug = true;
    let router = Router::new().route("/", get(|| async { "ok" }));

    let app = RuniqueApp::builder(config)
        .with_database(db)
        .routes(router)
        .static_files(|s| s.disable())
        .build()
        .await;

    assert!(app.is_ok(), "build() doit réussir avec SQLite in-memory");
}

#[tokio::test]
async fn test_builder_build_retourne_runique_app() {
    let db = Database::connect("sqlite::memory:").await.unwrap();
    let mut config = RuniqueConfig::from_env();
    config.debug = true;

    let app = RuniqueApp::builder(config)
        .with_database(db)
        .static_files(|s| s.disable())
        .build()
        .await
        .unwrap();

    // L'engine doit être présent
    assert!(
        app.engine.config.static_files.static_url.is_empty()
            || !app.engine.config.static_files.static_url.is_empty()
    );
}
