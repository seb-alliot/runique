//! `RuniqueEngine` implementation — construction, middleware attachment, store access.
use crate::middleware::session::{CleaningMemoryStore, session_db::RuniqueSessionStore};
use crate::utils::aliases::{
    ADb, ARlockmap, ASecurityCsp, ASecurityHosts, ATera, new, new_registry,
};
use axum::{Router, middleware};
use std::sync::{Arc, LazyLock, RwLock};
use tera::Tera;

use crate::config::RuniqueConfig;
// Import our newly renamed structures
use crate::middleware::{
    HostPolicy, MiddlewareConfig, SecurityPolicy, allowed_hosts_middleware, csrf_middleware,
    dev_no_cache_middleware, error_handler_middleware, https_redirect_middleware,
    security_headers_middleware,
};

#[cfg(feature = "orm")]
use sea_orm::DatabaseConnection;
/// Central machine of the framework: groups config, template engine,
/// database, URL registry, and all security policies.
#[derive(Debug)]
pub struct RuniqueEngine {
    /// General application configuration.
    pub config: RuniqueConfig,
    /// Shared Tera instance for read/write.
    pub tera: ATera,
    #[cfg(feature = "orm")]
    /// Database connection (feature `orm`).
    pub db: ADb,
    /// Global registry of named routes (reverse URL).
    pub url_registry: ARlockmap,
    /// Middleware toggles (cache, CSP, CSRF, etc.).
    pub features: MiddlewareConfig,
    /// Active Content Security Policy.
    pub security_csp: ASecurityCsp,
    /// Policy for validating allowed hosts.
    pub security_hosts: ASecurityHosts,
    /// Memory store — anonymous sessions + CSRF.
    pub session_store: LazyLock<RwLock<Option<Arc<CleaningMemoryStore>>>>,
    /// DB store — persistent authenticated sessions (table `eihwaz_sessions`).
    pub session_db_store: LazyLock<RwLock<Option<Arc<RuniqueSessionStore>>>>,
}

impl RuniqueEngine {
    /// Constructs a new engine from config, Tera, and DB connection.
    #[cfg(feature = "orm")]
    pub fn new(config: RuniqueConfig, tera: Tera, db: DatabaseConnection) -> Self {
        // Single load at startup
        let features = MiddlewareConfig::from_env();
        let security_csp = SecurityPolicy::default();
        let security_hosts = HostPolicy::default();

        Self {
            config,
            tera: new(tera),
            db: new(db),
            url_registry: new_registry(),
            features,
            security_csp: new(security_csp),
            security_hosts: new(security_hosts),
            session_store: LazyLock::new(|| RwLock::new(None)),
            session_db_store: LazyLock::new(|| RwLock::new(None)),
        }
    }

    /// Attaches global middlewares (HTTPS, hosts, CSRF, cache, CSP, errors)
    /// to the router based on active configuration.
    pub fn attach_middlewares(engine: Arc<Self>, router: Router) -> Router {
        let mut router = router;
        let f = &engine.features;

        // 0. HTTPS Redirection (First, to avoid unnecessary redirections)
        if engine.config.security.enforce_https {
            router = router.layer(middleware::from_fn_with_state(
                engine.clone(),
                https_redirect_middleware,
            ));
        }

        // 1. Host Validation (The very first line of defense)
        if f.enable_host_validation {
            router = router.layer(middleware::from_fn_with_state(
                engine.clone(),
                allowed_hosts_middleware,
            ));
        }

        // 2. CSRF (Security by design: integrated via ExtractForm + validation signal)
        // Note: We keep the middleware if you have global logic,
        // otherwise ExtractForm handles it as planned.
        router = router.layer(middleware::from_fn_with_state(
            engine.clone(),
            csrf_middleware,
        ));

        // 3. Cache (activated via .env)
        if !f.enable_cache {
            router = router.layer(middleware::from_fn_with_state(
                engine.clone(),
                dev_no_cache_middleware,
            ));
        }

        // 4. Security Headers (CSP, HSTS, etc.)
        if f.enable_csp {
            router = router.layer(middleware::from_fn_with_state(
                engine.clone(),
                security_headers_middleware,
            ));
        }

        // 5. Error Handler (Last, to catch errors from others)
        if f.enable_debug_errors {
            router = router.layer(middleware::from_fn(error_handler_middleware));
        }

        router
    }
}
