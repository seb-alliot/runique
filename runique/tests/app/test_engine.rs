//! Tests — engine/core.rs : RuniqueEngine::attach_middlewares()

use axum::Router;
use runique::engine::RuniqueEngine;

use crate::helpers::server::build_engine;

// ═══════════════════════════════════════════════════════════════
// attach_middlewares() — smoke tests
// ═══════════════════════════════════════════════════════════════

#[tokio::test]
async fn test_attach_middlewares_default_config() {
    let engine = build_engine().await;
    let router = RuniqueEngine::attach_middlewares(engine, Router::new());
    // Si ça compile et ne panique pas, la pile middleware est valide
    drop(router);
}

#[tokio::test]
async fn test_attach_middlewares_with_host_validation_enabled() {
    use runique::config::app::RuniqueConfig;
    use runique::middleware::config::MiddlewareConfig;
    use runique::middleware::security::{allowed_hosts::HostPolicy, csp::SecurityPolicy};
    use sea_orm::Database;
    use std::sync::Arc;
    use tera::Tera;

    let db = Database::connect("sqlite::memory:").await.unwrap();
    let mut config = RuniqueConfig::default();
    config.server.secret_key = "test_secret".to_string();

    let mut features = MiddlewareConfig::default();
    features.enable_host_validation = true;
    features.enable_csp = true;
    features.enable_debug_errors = true;

    let engine = Arc::new(RuniqueEngine {
        config,
        tera: Arc::new(Tera::default()),
        db: Arc::new(db),
        url_registry: Arc::new(std::sync::RwLock::new(std::collections::HashMap::new())),
        features,
        security_csp: Arc::new(SecurityPolicy::default()),
        security_hosts: Arc::new(HostPolicy::new(vec!["localhost".to_string()], false)),
        session_store: std::sync::LazyLock::new(|| std::sync::RwLock::new(None)),
        session_db_store: std::sync::LazyLock::new(|| std::sync::RwLock::new(None)),
    });

    let router = RuniqueEngine::attach_middlewares(engine, Router::new());
    drop(router);
}

#[tokio::test]
async fn test_attach_middlewares_with_https_redirect() {
    use runique::config::app::RuniqueConfig;
    use runique::middleware::config::MiddlewareConfig;
    use runique::middleware::security::{allowed_hosts::HostPolicy, csp::SecurityPolicy};
    use sea_orm::Database;
    use std::sync::Arc;
    use tera::Tera;

    let db = Database::connect("sqlite::memory:").await.unwrap();
    let mut config = RuniqueConfig::default();
    config.server.secret_key = "test_secret".to_string();
    config.security.enforce_https = true;

    let engine = Arc::new(RuniqueEngine {
        config,
        tera: Arc::new(Tera::default()),
        db: Arc::new(db),
        url_registry: Arc::new(std::sync::RwLock::new(std::collections::HashMap::new())),
        features: MiddlewareConfig::default(),
        security_csp: Arc::new(SecurityPolicy::default()),
        security_hosts: Arc::new(HostPolicy::new(vec![], true)),
        session_store: std::sync::LazyLock::new(|| std::sync::RwLock::new(None)),
        session_db_store: std::sync::LazyLock::new(|| std::sync::RwLock::new(None)),
    });

    let router = RuniqueEngine::attach_middlewares(engine, Router::new());
    drop(router);
}
