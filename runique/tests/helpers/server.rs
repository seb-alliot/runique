//! Shared test server builder for integration tests.
//!
//! Starts a real axum server once (via OnceLock + dedicated thread) and
//! returns its SocketAddr to any test that calls `test_server_addr()`.
//!
//! Usage:
//! ```rust
//! use crate::helpers::server::test_server_addr;
//!
//! #[tokio::test]
//! async fn my_test() {
//!     let addr = test_server_addr();
//!     let resp = reqwest::get(format!("http://{}/", addr)).await.unwrap();
//!     assert_eq!(resp.status(), 200);
//! }
//! ```

use axum::{
    middleware,
    routing::{delete, get, post},
    Router,
};
use runique::{
    config::app::RuniqueConfig,
    engine::RuniqueEngine,
    middleware::{
        config::MiddlewareConfig,
        security::{allowed_hosts::HostPolicy, csp::SecurityPolicy, csrf::csrf_middleware},
    },
};
use sea_orm::Database;
use std::{net::SocketAddr, sync::Arc, sync::OnceLock};
use tera::Tera;
use tower_sessions::{MemoryStore, SessionManagerLayer};

// ── Constants ─────────────────────────────────────────────────────────────────

/// SQLite in-memory connection URL used across all test helpers.
pub const SQLITE_URL: &str = "sqlite::memory:";

/// Secret key used by the test server (deterministic for reproducible tokens).
pub const TEST_SECRET: &str = "runique_test_secret_key_for_integration";

// ── Shared server ─────────────────────────────────────────────────────────────

static SERVER_ADDR: OnceLock<SocketAddr> = OnceLock::new();

/// Returns the address of the shared test server, starting it on first call.
///
/// The server runs in its own thread + runtime so it is independent of the
/// tokio runtime created by each `#[tokio::test]`.
pub fn test_server_addr() -> SocketAddr {
    *SERVER_ADDR.get_or_init(|| {
        let (tx, rx) = std::sync::mpsc::channel();

        std::thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().expect("test runtime");
            rt.block_on(async {
                let engine = build_engine().await;
                let app = build_default_router(engine);

                let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
                    .await
                    .expect("bind test server");
                let addr = listener.local_addr().expect("local addr");

                tx.send(addr).expect("send addr");
                axum::serve(listener, app).await.expect("serve");
            });
        });

        rx.recv().expect("recv addr")
    })
}

// ── Engine builder ─────────────────────────────────────────────────────────────

/// Builds a `RuniqueEngine` with SQLite in-memory and default middleware config.
/// Reuse this in tests that need `oneshot()` rather than a persistent server.
pub async fn build_engine() -> Arc<RuniqueEngine> {
    let db = Database::connect(SQLITE_URL)
        .await
        .expect("sqlite::memory: connect");

    let mut config = RuniqueConfig::default();
    config.server.secret_key = TEST_SECRET.to_string();

    Arc::new(RuniqueEngine {
        config,
        tera: Arc::new(Tera::default()),
        db: Arc::new(db),
        url_registry: Arc::new(std::sync::RwLock::new(std::collections::HashMap::new())),
        features: MiddlewareConfig::default(),
        security_csp: Arc::new(SecurityPolicy::default()),
        security_hosts: Arc::new(HostPolicy::new(vec![], true)),
    })
}

// ── Default router ─────────────────────────────────────────────────────────────

/// Builds the default test Router: CSRF middleware + session layer.
///
/// Routes:
///   GET  /         → 200 "ok"
///   POST /submit   → 200 "submitted"
///   DELETE /delete → 200 "deleted"
pub fn build_default_router(engine: Arc<RuniqueEngine>) -> Router {
    let session_layer = SessionManagerLayer::new(MemoryStore::default());

    Router::new()
        .route("/", get(|| async { "ok" }))
        .route("/submit", post(|| async { "submitted" }))
        .route("/delete", delete(|| async { "deleted" }))
        .layer(middleware::from_fn_with_state(engine, csrf_middleware))
        .layer(session_layer)
}

/// Builds a `reqwest::Client` pre-configured for the test server.
pub fn test_client() -> reqwest::Client {
    reqwest::Client::builder()
        .cookie_store(true) // preserve session cookies across requests
        .build()
        .expect("reqwest client")
}
