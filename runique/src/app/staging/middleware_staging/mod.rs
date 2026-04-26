//! Middleware staging: configuration API for the middleware stack.
mod applicator;

use super::csp_config::CspConfig;
use super::host_config::HostConfig;
use crate::app::error_build::BuildError;
use crate::config::RuniqueConfig;
use crate::middleware::{MiddlewareConfig, SecurityPolicy};
use axum::Router;
use tower_sessions::cookie::time::Duration;
use tower_sessions::{Expiry, SessionManagerLayer, SessionStore};

// ─── Internal type aliases ────────────────────────────────────────────────────

/// Type-erased closure for a custom session store.
/// Params: (Router, debug: bool, duration: Duration) -> Router
pub(crate) type SessionApplicator = Box<dyn FnOnce(Router, bool, Duration) -> Router + Send>;

/// Type-erased closure for a developer's custom middleware.
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
        let config: HostConfig = f(HostConfig::default());
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
}
