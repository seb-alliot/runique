//! Middleware staging: automatic reorganization by slots and application to the router.
use super::csp_config::CspConfig;
use super::host_config::HostConfig;
use crate::app::error_build::BuildError;
use crate::config::RuniqueConfig;
use crate::context::RequestExtensions;
use crate::middleware::session::CleaningMemoryStore;
use crate::middleware::{
    MiddlewareConfig, SecurityPolicy, allowed_hosts_middleware, csp_middleware, csrf_middleware,
    dev_no_cache_middleware, error_handler_middleware, security_headers_middleware,
};
use crate::utils::aliases::{AEngine, ARuniqueConfig, ATera};
use axum::{self, Router, middleware};
use std::sync::Arc;
use tower_http::compression::CompressionLayer;
use tower_sessions::cookie::time::Duration;
use tower_sessions::{Expiry, SessionManagerLayer, SessionStore};

// ═══════════════════════════════════════════════════════════════
// MiddlewareStaging — Automatic Reorganization by Slots
// ═══════════════════════════════════════════════════════════════
//
// Key Runique innovation:
// The developer configures their middlewares in the order they want.
// Each middleware has a fixed SLOT (priority).
// At build time, staging sorts by slot and applies in the
// optimal order — automatically.
//
// CSRF reads/writes a token in the session → it DEPENDS on Session.
// With raw Axum, if you put CSRF before Session → silent bug.
// With Runique → it works anyway, the framework reorders.
//
// ═══════════════════════════════════════════════════════════════
//
// AXUM MODEL:
//   .layer(A).layer(B).layer(C)
//   Request execution: C → B → A → Handler
//   Last added `.layer()` = outermost = first executed
//
// OUR STRATEGY:
//   Low slot (0)   = external = first executed on the request
//   High slot (200+) = internal = closer to the handler
//
//   We sort DESCENDING then apply `.layer()` in this order:
//   the highest slot is applied FIRST (.layer) = the most INTERNAL
//   the lowest slot is applied LAST (.layer) = the most EXTERNAL
//
// RESULT on an incoming request:
//   → Extensions(0) → ErrorHandler(10) → Custom(20+)
//   → CSP(30) → Cache(40) → Session(50) → CSRF(60)
//   → Host(70) → Handler
//
// Reproduces the proven order of the old builder:
//   ErrorHandler wraps EVERYTHING → catches all errors
//   ErrorHandler extracts Extension(tera/config) → injected by Extensions
//   Session executed BEFORE CSRF → CSRF can read the session
//   Host = last defense before the handler
//
// ═══════════════════════════════════════════════════════════════

// ─────────────────────────────────────────────────────────────
// Built-in slots — Guaranteed execution order on the request
// ─────────────────────────────────────────────────────────────

const SLOT_EXTENSIONS: u16 = 0; // Engine/Tera/Config injection (outermost)
const SLOT_COMPRESSION: u16 = 5; // Compression (external, before any other middleware)
const SLOT_ERROR_HANDLER: u16 = 10; // Catches errors of the WHOLE stack
const SLOT_SECURITY_HEADERS: u16 = 30; // security headers
const SLOT_SECURITY_CSP: u16 = 31;
const SLOT_CACHE: u16 = 40; // Cache headers
const SLOT_SESSION: u16 = 50; // Before CSRF (CSRF depends on it)
const SLOT_SESSION_UPGRADE: u16 = 55; // After Session (reads/writes in session)
const SLOT_AUTH: u16 = 57; // After Session — loads CurrentUser from the session
const SLOT_CSRF: u16 = 60; // After Session (reads/writes in session)
const SLOT_HOST_VALIDATION: u16 = 70; // Last defense before handler

// Dev's custom middlewares start HERE
// Placed between ErrorHandler and CSP → wrapped by ErrorHandler
const SLOT_CUSTOM_BASE: u16 = 20;

// ─────────────────────────────────────────────────────────────
// MiddlewareEntry — A middleware with its priority slot
// ─────────────────────────────────────────────────────────────

struct MiddlewareEntry {
    /// Slot = position in the stack.
    /// Low (0) = external, first executed.
    /// High (100+) = internal, close to the handler.
    slot: u16,

    /// Human-readable name for debug and logs
    #[allow(dead_code)]
    name: &'static str,

    /// Type-erased closure that applies the middleware on the router
    apply: Box<dyn FnOnce(Router) -> Router + Send>,
}

// ─────────────────────────────────────────────────────────────
// Internal types
// ─────────────────────────────────────────────────────────────

/// Type-erased closure for a custom session store
/// Params: (Router, debug: bool, duration: Duration) -> Router
pub(crate) type SessionApplicator = Box<dyn FnOnce(Router, bool, Duration) -> Router + Send>;

/// Type-erased closure for a developer's custom middleware
pub(crate) type CustomMiddleware = Box<dyn FnOnce(Router) -> Router + Send>;

// ═══════════════════════════════════════════════════════════════
// MiddlewareStaging
// ═══════════════════════════════════════════════════════════════

pub struct MiddlewareStaging {
    /// Middleware features configuration (CSP, Host, Cache, etc.)
    pub(crate) features: MiddlewareConfig,

    /// Inactivity duration before session expiration
    pub(crate) session_duration: Duration,

    /// Anonymous session duration => anonymous session lifetime
    pub(crate) anonymous_session_duration: Duration,
    /// Custom session applicator (None = default MemoryStore)
    pub(crate) session_applicator: Option<SessionApplicator>,

    /// `CleaningMemoryStore` memory watermarks (low, high) in bytes
    pub(crate) session_low_watermark: usize,
    pub(crate) session_high_watermark: usize,

    /// Periodic cleanup interval in seconds (default: 60s, via `RUNIQUE_SESSION_CLEANUP_SECS`)
    pub(crate) session_cleanup_interval_secs: u64,

    /// Only one device connected at a time per user (default: false)
    pub(crate) exclusive_login: bool,

    /// Developer's custom middlewares (order of addition preserved)
    pub(crate) custom_middlewares: Vec<CustomMiddleware>,

    /// CSP policy defined via the builder (None = read from `.env`)
    pub(crate) security_policy: Option<SecurityPolicy>,

    /// Allowed hosts defined via the builder
    pub(crate) allowed_hosts: Vec<String>,
}

impl MiddlewareStaging {
    /// Creates a `MiddlewareStaging` adapted to the mode (debug/production)
    pub fn new(debug: bool) -> Self {
        let features = if debug {
            MiddlewareConfig::development()
        } else {
            MiddlewareConfig::production()
        };

        Self {
            features,
            session_duration: Duration::seconds(86400),
            anonymous_session_duration: Duration::seconds(300),
            session_applicator: None,
            session_low_watermark: 128 * 1024 * 1024,
            session_high_watermark: 256 * 1024 * 1024,
            session_cleanup_interval_secs: 60,
            exclusive_login: false,
            custom_middlewares: Vec::new(),
            security_policy: None,
            allowed_hosts: Vec::new(),
        }
    }

    /// Creates a `MiddlewareStaging` from `RuniqueConfig`.
    ///
    /// Resolution strategy:
    ///   1. `RUNIQUE_ENABLE_*` variables from `.env` take priority
    ///   2. If absent, debug mode determines defaults:
    ///      - debug=true  → `development()` profile (permissive)
    ///      - debug=false → `production()` profile (strict)
    ///
    /// The dev can then override via `.middleware(|m| m.with_csp(true))`.
    pub fn from_config(config: &RuniqueConfig) -> Self {
        // Base profile according to mode
        let defaults = if config.debug {
            MiddlewareConfig::development()
        } else {
            MiddlewareConfig::production()
        };

        // .env variables take priority over the profile
        let get_env_or = |key: &str, default: bool| -> bool {
            std::env::var(key)
                .map(|v| v.parse::<bool>().unwrap_or(default))
                .unwrap_or(default)
        };

        let features = MiddlewareConfig {
            // CSP configured only via the builder (.with_csp(true/false))
            enable_csp: false,
            enable_header_security: false,
            // host validation configured only via the builder (.with_allowed_hosts)
            enable_host_validation: false,
            enable_debug_errors: true, // always mounted — config.debug manages the content
            enable_cache: get_env_or("RUNIQUE_ENABLE_CACHE", defaults.enable_cache),
            exclusive_login: false, // propagated via `apply_to_router` from `self.exclusive_login`
        };

        Self {
            features,
            session_duration: Duration::seconds(86400),
            anonymous_session_duration: Duration::seconds(300),
            session_applicator: None,
            session_low_watermark: 128 * 1024 * 1024,
            session_high_watermark: 256 * 1024 * 1024,
            session_cleanup_interval_secs: 60,
            exclusive_login: false,
            custom_middlewares: Vec::new(),
            security_policy: None,
            allowed_hosts: Vec::new(),
        }
    }

    // ═══════════════════════════════════════════════════
    // Features configuration
    // ═══════════════════════════════════════════════════

    /// Configures the Content Security Policy via a closure.
    ///
    /// The closure receives a [`CspConfig`] and returns the configured `CspConfig`.
    /// Everything is disabled by default — explicitly enable what you need.
    /// To disable CSP: do not call `.with_csp` at all.
    ///
    /// # Example — custom configuration
    /// ```rust,ignore
    /// .middleware(|m| {
    ///     m.with_csp(|c| {
    ///         c.with_header_security(true)
    ///          .with_nonce(true)
    ///          .scripts(vec!["'self'", "https://cdn.jsdelivr.net"])
    ///          .styles(vec!["'self'", "https://cdn.jsdelivr.net"])
    ///          .images(vec!["'self'", "data:"])
    ///     })
    /// })
    /// ```
    ///
    /// # Example — strict preset
    /// ```rust,ignore
    /// use runique::middleware::SecurityPolicy;
    ///
    /// .middleware(|m| {
    ///     m.with_csp(|c| {
    ///         c.policy(SecurityPolicy::strict())
    ///          .with_header_security(true)
    ///     })
    /// })
    /// ```
    ///
    /// # Example — CSP with defaults only
    /// ```rust,ignore
    /// .middleware(|m| m.with_csp(|c| c))
    /// ```
    pub fn with_csp(mut self, f: impl FnOnce(CspConfig) -> CspConfig) -> Self {
        let csp = f(CspConfig::default());
        self.features.enable_csp = true;
        self.features.enable_header_security = csp.enable_header_security;
        self.security_policy = Some(csp.policy);
        self
    }

    /// Configures allowed hosts validation via a closure.
    ///
    /// The closure receives a [`HostConfig`] and returns the configured `HostConfig`.
    /// Validation is **disabled by default** — call `.enabled(true)` explicitly.
    /// To disable: do not call `.with_allowed_hosts` at all.
    ///
    /// # Example
    /// ```rust,ignore
    /// .middleware(|m| {
    ///     m.with_allowed_hosts(|h| {
    ///         h.enabled(true)
    ///          .host("mysite.com")
    ///          .host("www.mysite.com")
    ///     })
    /// })
    /// ```
    ///
    /// # Wildcard subdomains
    /// ```rust,ignore
    /// m.with_allowed_hosts(|h| {
    ///     h.enabled(true).host(".mysite.com")
    /// })
    /// ```
    pub fn with_allowed_hosts(mut self, f: impl FnOnce(HostConfig) -> HostConfig) -> Self {
        let config = f(HostConfig::default());
        self.allowed_hosts = config.hosts;
        self.features.enable_host_validation = config.enabled;
        self
    }

    /// Enables or disables debug error pages
    pub fn with_debug_errors(mut self, enable: bool) -> Self {
        self.features.enable_debug_errors = enable;
        self
    }

    /// Enables or disables HTTP cache
    pub fn with_cache(mut self, enable: bool) -> Self {
        self.features.enable_cache = enable;
        self
    }

    // ═══════════════════════════════════════════════════
    // Session configuration
    // ═══════════════════════════════════════════════════

    /// Configures the inactivity duration before session expiration
    pub fn with_session_duration(mut self, duration: Duration) -> Self {
        self.session_duration = duration;
        self
    }

    /// Configures the inactivity duration before anonymous session expiration
    pub fn with_anonymous_session_duration(mut self, duration: Duration) -> Self {
        self.anonymous_session_duration = duration;
        self
    }

    /// Configures the `CleaningMemoryStore` memory watermarks.
    ///
    /// - `low`: triggers a proactive (non-blocking) cleanup of expired anonymous sessions
    /// - `high`: synchronous emergency cleanup + refusal if still exceeded (503)
    ///
    pub fn with_session_memory_limit(mut self, low: usize, high: usize) -> Self {
        self.session_low_watermark = low;
        self.session_high_watermark = high;
        self
    }

    /// Configures the periodic cleanup interval.
    ///
    pub fn with_session_cleanup_interval(mut self, secs: u64) -> Self {
        self.session_cleanup_interval_secs = secs;
        self
    }

    /// Enables or disables exclusive login (only one device at a time).
    ///
    /// Defaults to `false`. If `true`, any new connection automatically invalidates
    /// existing sessions of the same user — without modifying handlers.
    ///
    /// # Example
    /// ```rust,ignore
    /// .middleware(|m| m.with_exclusive_login(true))
    /// ```
    pub fn with_exclusive_login(mut self, exclusive: bool) -> Self {
        self.exclusive_login = exclusive;
        self
    }

    /// Configures a custom session store (Redis, PostgreSQL, etc.)
    ///
    /// # Example
    /// ```rust,ignore
    /// .middleware(|m| {
    ///     m.with_session_store(RedisStore::new(client))
    ///      .with_session_duration(Duration::hours(2))
    /// })
    /// ```
    pub fn with_session_store<S: SessionStore + Clone + Send + Sync + 'static>(
        mut self,
        store: S,
    ) -> Self {
        self.session_applicator = Some(Box::new(
            move |router: Router, debug: bool, duration: Duration| {
                let layer = SessionManagerLayer::new(store)
                    .with_secure(!debug)
                    .with_http_only(true)
                    .with_same_site(tower_sessions::cookie::SameSite::Strict)
                    .with_expiry(Expiry::OnInactivity(duration));
                router.layer(layer)
            },
        ));
        self
    }

    // ═══════════════════════════════════════════════════
    // Developer's custom middlewares
    // ═══════════════════════════════════════════════════

    /// Adds a custom middleware.
    ///
    /// Automatic position: `len + 1` — always AFTER all
    /// built-in middlewares, closest to the handler.
    ///
    /// If multiple customs are added, they are placed in
    /// the order of addition (slot 100, 101, 102...).
    ///
    /// # Example
    /// ```rust,ignore
    /// .middleware(|m| {
    ///     m.add_custom(|router| {
    ///         router.layer(my_auth_layer)
    ///     })
    /// })
    /// ```
    pub fn add_custom(mut self, mw: impl FnOnce(Router) -> Router + Send + 'static) -> Self {
        self.custom_middlewares.push(Box::new(mw));
        self
    }

    // ═══════════════════════════════════════════════════
    // Validation
    // ═══════════════════════════════════════════════════

    /// Validates the consistency of the middleware configuration
    pub fn validate(&self) -> Result<(), BuildError> {
        // CSRF always enabled → nothing to validate
        //
        // Future validations:
        // - host_validation enabled → ALLOWED_HOSTS defined?
        // - enable_debug_errors in production → warning
        Ok(())
    }

    /// Middlewares are always ready
    pub fn is_ready(&self) -> bool {
        true
    }

    /// Returns the active middleware features configuration
    pub fn features(&self) -> &MiddlewareConfig {
        &self.features
    }

    /// Returns the list of configured allowed hosts
    pub fn allowed_hosts(&self) -> &[String] {
        &self.allowed_hosts
    }

    /// Returns the configured session duration
    pub fn session_duration(&self) -> Duration {
        self.session_duration
    }

    /// Returns the number of custom middlewares added
    pub fn custom_count(&self) -> usize {
        self.custom_middlewares.len()
    }

    // ═══════════════════════════════════════════════════════════
    // APPLICATION — The heart of innovation
    //
    // Builds the full middleware stack:
    // 1. Collects all entries (built-in + custom)
    // 2. Each entry has a fixed slot
    // 3. DESCENDING sort by slot
    // 4. `.layer()` application in this order
    //
    // Result (execution on request):
    //   Extensions → ErrorHandler → Custom → CSP → Cache
    //   → Session → CSRF → Host → Handler
    // ═══════════════════════════════════════════════════════════

    pub(crate) fn apply_to_router(
        self,
        router: Router,
        config: ARuniqueConfig,
        engine: AEngine,
        tera: ATera,
    ) -> (Router, Option<Arc<CleaningMemoryStore>>) {
        let debug = config.debug;
        let mut entries: Vec<MiddlewareEntry> = Vec::new();

        // ═══════════════════════════════════════
        // BUILT-IN: each middleware has a fixed slot.
        // Regardless of whether the dev enables CSP before Host,
        // sorting by slot guarantees the correct order.
        // ═══════════════════════════════════════

        // Slot 0: Extensions (Engine, Tera, Config) — outermost
        {
            let eng = engine.clone();
            let t = tera.clone();
            let c = config.clone();
            entries.push(MiddlewareEntry {
                slot: SLOT_EXTENSIONS,
                name: "Extensions",
                apply: Box::new(move |r| {
                    r.layer(axum::middleware::from_fn(
                        move |mut req: axum::http::Request<axum::body::Body>,
                              next: axum::middleware::Next| {
                            let extensions = RequestExtensions::new()
                                .with_tera(t.clone())
                                .with_config(c.clone())
                                .with_engine(eng.clone());
                            extensions.inject_request(&mut req);
                            async move { next.run(req).await }
                        },
                    ))
                }),
            });
        }
        // Slot 5: Compression — before any other middleware
        {
            entries.push(MiddlewareEntry {
                slot: SLOT_COMPRESSION,
                name: "Compression",
                apply: Box::new(|r| r.layer(CompressionLayer::new())),
            });
        }
        // Slot 70: Host validation — last defense before handler
        if self.features.enable_host_validation {
            let eng = engine.clone();
            entries.push(MiddlewareEntry {
                slot: SLOT_HOST_VALIDATION,
                name: "HostValidation",
                apply: Box::new(move |r| {
                    r.layer(middleware::from_fn_with_state(
                        eng,
                        allowed_hosts_middleware,
                    ))
                }),
            });
        }

        // Slot 50: Session — before CSRF (CSRF depends on it)
        let memory_store: Option<Arc<CleaningMemoryStore>> = {
            let applicator = self.session_applicator;
            let anon_duration = self.anonymous_session_duration;
            let low_wm = self.session_low_watermark;
            let high_wm = self.session_high_watermark;
            let cleanup_secs = self.session_cleanup_interval_secs;

            let exclusive_login = self.exclusive_login;
            let store_arc = if applicator.is_none() {
                let store = Arc::new(
                    CleaningMemoryStore::default()
                        .with_watermarks(low_wm, high_wm)
                        .with_exclusive_login(exclusive_login),
                );
                store.spawn_cleanup(tokio::time::Duration::from_secs(cleanup_secs));
                Some(store)
            } else {
                None
            };

            let store_for_layer = store_arc.clone();
            entries.push(MiddlewareEntry {
                slot: SLOT_SESSION,
                name: "Session",
                apply: Box::new(move |r: Router| match applicator {
                    Some(apply_fn) => apply_fn(r, debug, anon_duration),
                    None => {
                        let store = store_for_layer.expect("store created above");
                        let layer = SessionManagerLayer::new((*store).clone())
                            .with_secure(!debug)
                            .with_http_only(true)
                            .with_same_site(tower_sessions::cookie::SameSite::Strict)
                            .with_expiry(Expiry::OnInactivity(anon_duration));
                        r.layer(layer)
                    }
                }),
            });

            store_arc
        };

        // Slot 60: CSRF — ALWAYS enabled, after Session
        {
            let eng = engine.clone();
            entries.push(MiddlewareEntry {
                slot: SLOT_CSRF,
                name: "CSRF",
                apply: Box::new(move |r| {
                    r.layer(middleware::from_fn_with_state(eng, csrf_middleware))
                }),
            });
        }

        // Slot 40: Cache control
        if !self.features.enable_cache {
            let eng = engine.clone();
            entries.push(MiddlewareEntry {
                slot: SLOT_CACHE,
                name: "NoCache",
                apply: Box::new(move |r| {
                    r.layer(middleware::from_fn_with_state(eng, dev_no_cache_middleware))
                }),
            });
        }

        // Slot 30: Security headers — ALWAYS active
        {
            let eng = engine.clone();
            entries.push(MiddlewareEntry {
                slot: SLOT_SECURITY_HEADERS,
                name: "SecurityHeaders",
                apply: Box::new(move |r| {
                    r.layer(middleware::from_fn_with_state(
                        eng,
                        security_headers_middleware,
                    ))
                }),
            });
        }

        // Slot 31: CSP — only if enabled
        if self.features.enable_csp {
            let eng = engine.clone();
            entries.push(MiddlewareEntry {
                slot: SLOT_SECURITY_CSP,
                name: "CSP",
                apply: Box::new(move |r| {
                    r.layer(middleware::from_fn_with_state(eng, csp_middleware))
                }),
            });
        }

        // Slot 55: Upgrade TTL if authenticated
        {
            entries.push(MiddlewareEntry {
                slot: SLOT_SESSION_UPGRADE,
                name: "SessionTtlUpgrade",
                apply: Box::new(move |r| {
                    r.layer(axum::middleware::from_fn(
                        move |req: axum::http::Request<axum::body::Body>,
                              next: axum::middleware::Next| {
                            async move {
                                if let Some(session) =
                                    req.extensions().get::<tower_sessions::Session>()
                                {
                                    if crate::auth::session::is_authenticated(session).await {
                                        session.set_expiry(Some(Expiry::OnInactivity(
                                            self.session_duration,
                                        )));
                                    }
                                }
                                next.run(req).await
                            }
                        },
                    ))
                }),
            });
        }

        // Slot 57: Auth — loads `CurrentUser` from the session and injects it into extensions
        {
            entries.push(MiddlewareEntry {
                slot: SLOT_AUTH,
                name: "Auth",
                apply: Box::new(|r| {
                    r.layer(axum::middleware::from_fn(
                        |mut req: axum::http::Request<axum::body::Body>,
                         next: axum::middleware::Next| async move {
                            use crate::admin::permissions::Groupe;
                            use crate::auth::session::{CurrentUser, get_user_id, get_username};
                            use crate::utils::constante::{
                                admin_context::permission::GROUPES,
                                session_key::session::{
                                    SESSION_USER_IS_STAFF_KEY, SESSION_USER_IS_SUPERUSER_KEY,
                                },
                            };
                            if let Some(session) =
                                req.extensions().get::<tower_sessions::Session>().cloned()
                            {
                                if let (Some(id), Some(username)) =
                                    (get_user_id(&session).await, get_username(&session).await)
                                {
                                    let is_staff = session
                                        .get::<bool>(SESSION_USER_IS_STAFF_KEY)
                                        .await
                                        .ok()
                                        .flatten()
                                        .unwrap_or(false);
                                    let is_superuser = session
                                        .get::<bool>(SESSION_USER_IS_SUPERUSER_KEY)
                                        .await
                                        .ok()
                                        .flatten()
                                        .unwrap_or(false);
                                    let groupes = session
                                        .get::<Vec<Groupe>>(GROUPES)
                                        .await
                                        .ok()
                                        .flatten()
                                        .unwrap_or_default();
                                    let current_user = CurrentUser {
                                        id,
                                        username,
                                        is_staff,
                                        is_superuser,
                                        groupes,
                                    };
                                    RequestExtensions::new()
                                        .with_current_user(current_user)
                                        .inject_request(&mut req);
                                }
                            }
                            next.run(req).await
                        },
                    ))
                }),
            });
        }

        // Slot 10: Error handler — wraps the WHOLE stack, catches all errors
        // Extracts Extension(tera) and Extension(config) injected by Extensions (slot 0)
        if self.features.enable_debug_errors {
            entries.push(MiddlewareEntry {
                slot: SLOT_ERROR_HANDLER,
                name: "ErrorHandler",
                apply: Box::new(|r| r.layer(middleware::from_fn(error_handler_middleware))),
            });
        }

        // ═══════════════════════════════════════
        // CUSTOM: Automatic position between ErrorHandler and CSP.
        //
        // The dev doesn't choose a slot.
        // Their middlewares are wrapped by ErrorHandler
        // but executed before security middlewares.
        //
        // First custom → slot 20
        // Second custom → slot 21
        // etc.
        // ═══════════════════════════════════════

        for (i, custom_mw) in self.custom_middlewares.into_iter().enumerate() {
            entries.push(MiddlewareEntry {
                slot: SLOT_CUSTOM_BASE.saturating_add(i as u16),
                name: "Custom",
                apply: custom_mw,
            });
        }

        // ═══════════════════════════════════════
        // DESCENDING SORT + APPLICATION
        //
        // Highest slot → first `.layer()` → most INTERNAL
        // Lowest slot  → last `.layer()` → most EXTERNAL
        //
        // In Axum: last `.layer()` = first executed on the request
        //
        // Result on the request:
        //   Extensions(0) → ErrorHandler(10) → Custom(20+)
        //   → CSP(30) → Cache(40) → Session(50) → CSRF(60)
        //   → Host(70) → Handler
        //
        // ErrorHandler wraps everything → catches all errors
        // Session before CSRF → CSRF can read the session
        // ═══════════════════════════════════════

        entries.sort_by_key(|b| std::cmp::Reverse(b.slot));

        let mut router = router;
        for entry in entries {
            router = (entry.apply)(router);
        }

        (router, memory_store)
    }
}
