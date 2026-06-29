//! `RuniqueEngine` implementation — construction, middleware attachment, store access.
use crate::middleware::session::{CleaningMemoryStore, session_db::RuniqueSessionStore};
use crate::utils::aliases::{
    ADb, ARlockmap, ASecurityCsp, ASecurityHosts, ATera, new, new_registry,
};
use axum::{Router, middleware};
use std::any::TypeId;
use std::collections::HashMap;
use std::sync::{Arc, LazyLock, RwLock};
use tera::Tera;

use crate::config::RuniqueConfig;
// Import our newly renamed structures
use crate::middleware::{
    HostPolicy, MiddlewareConfig, PermissionsPolicy, SecurityPolicy, TrustedProxies,
    allowed_hosts_middleware, csrf_middleware, dev_no_cache_middleware, error_handler_middleware,
    https_redirect_middleware, security_headers_middleware,
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
    /// Paths exempt from CSRF validation (ex: webhook endpoints).
    pub csrf_exempt_paths: Arc<Vec<String>>,
    /// Active Permissions-Policy header configuration.
    pub permissions_policy: Arc<PermissionsPolicy>,
    /// Trusted proxy IPs/CIDRs for real client IP extraction.
    pub trusted_proxies: Arc<TrustedProxies>,
    /// Memory store — anonymous sessions + CSRF.
    pub session_store: LazyLock<RwLock<Option<Arc<CleaningMemoryStore>>>>,
    /// DB store — persistent authenticated sessions (table `eihwaz_sessions`).
    pub session_db_store: LazyLock<RwLock<Option<Arc<RuniqueSessionStore>>>>,
    /// Extension map — custom external connections registered via `with_custom_db()`.
    /// Keyed by `TypeId`, supports multiple types simultaneously.
    pub extensions: HashMap<TypeId, Arc<dyn std::any::Any + Send + Sync>>,
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
            csrf_exempt_paths: Arc::new(vec![]),
            permissions_policy: Arc::new(PermissionsPolicy::default()),
            trusted_proxies: Arc::new(TrustedProxies::default()),
            session_store: LazyLock::new(|| RwLock::new(None)),
            session_db_store: LazyLock::new(|| RwLock::new(None)),
            extensions: HashMap::new(),
        }
    }

    /// `true` if the in-memory session store has reached its high watermark — a new
    /// session would be refused. Lets a login handler fail fast and clean (503 +
    /// `Retry-After`) instead of letting tower's commit-time `create` error surface
    /// as a generic 500. Returns `false` if the store is not yet initialized.
    ///
    /// # Examples
    /// ```rust,ignore
    /// // At the top of a public login handler:
    /// if engine.session_store_saturated() {
    ///     return StatusCode::SERVICE_UNAVAILABLE.into_response();
    /// }
    /// ```
    #[must_use]
    pub fn session_store_saturated(&self) -> bool {
        self.session_store
            .read()
            .ok()
            .and_then(|g| g.as_ref().map(|s| s.is_saturated()))
            .unwrap_or(false)
    }

    /// Retrieves a custom extension registered via `with_custom_db()`.
    ///
    /// Returns `Option<Arc<T>>` — `None` if this type was not registered.
    ///
    /// # Examples
    /// ```rust,ignore
    /// // In a handler:
    /// if let Some(mongo) = req.engine.extension::<mongodb::Client>() {
    ///     let col = mongo.database("mydb").collection::<Document>("items");
    /// }
    ///
    /// // Multiple types:
    /// let redis = req.engine.extension::<redis::Client>();
    /// let mongo = req.engine.extension::<mongodb::Client>();
    /// ```
    pub fn extension<T: std::any::Any + Send + Sync + 'static>(&self) -> Option<Arc<T>> {
        self.extensions
            .get(&TypeId::of::<T>())
            .and_then(|arc| arc.clone().downcast::<T>().ok())
    }

    /// Alias for [`extension`](Self::extension) — kept for backward compatibility.
    pub fn custom_db<T: std::any::Any + Send + Sync + 'static>(&self) -> Option<Arc<T>> {
        self.extension::<T>()
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
