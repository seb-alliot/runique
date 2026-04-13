//! Centralized configuration of Runique middlewares — session, CSP, CSRF, debug errors, rate limit.

/// Centralized configuration of all Runique middlewares
///
/// Note: CSRF is ALWAYS enabled (imposed by the framework for security)
///
/// ## Error page management
/// `enable_debug_errors` controls whether the `error_handler` middleware is added to the router.
/// When active, error pages are rendered by Tera:
/// - In **development** (`DEBUG=true` or `cargo build` without `--release`): full traces.
/// - In **production** (`cargo build --release`): clean 404/500 pages without traces.
///
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MiddlewareConfig {
    pub enable_csp: bool,
    /// Enables additional security headers (HSTS, X-Frame-Options, COEP, COOP, CORP,
    /// Referrer-Policy, Permissions-Policy). Has no effect if `enable_csp` is false.
    ///
    /// When `true`: uses `security_headers_middleware` (CSP + additional headers).
    /// When `false`: uses `csp_middleware` (CSP only).
    pub enable_header_security: bool,
    pub enable_host_validation: bool,
    /// Enables `error_handler` middleware which intercepts 4xx/5xx errors.
    /// Disable only if you handle errors manually in each handler.
    pub enable_debug_errors: bool,
    pub enable_cache: bool,
    pub exclusive_login: bool,
}

impl Default for MiddlewareConfig {
    fn default() -> Self {
        Self {
            enable_csp: true,
            enable_header_security: false,
            enable_host_validation: true,
            enable_debug_errors: true,
            enable_cache: true,
            exclusive_login: false,
        }
    }
}

impl MiddlewareConfig {
    pub fn from_env() -> Self {
        let get_bool = |key: &str, default: bool| {
            std::env::var(key)
                .map(|v| v.parse::<bool>().unwrap_or(default))
                .unwrap_or(default)
        };

        Self {
            // CSP and host validation configured only via the builder
            enable_csp: false,
            enable_header_security: false,
            enable_host_validation: false,
            enable_debug_errors: true, // always mounted — config.debug handles content
            enable_cache: get_bool("RUNIQUE_ENABLE_CACHE", true),
            exclusive_login: false,
        }
    }

    /// Configuration for production (maximum security)
    pub fn production() -> Self {
        Self {
            enable_csp: true,
            enable_header_security: false,
            enable_host_validation: true,
            enable_debug_errors: true,
            enable_cache: true,
            exclusive_login: false,
        }
    }

    /// Configuration for development (more permissive)
    pub fn development() -> Self {
        Self {
            enable_csp: false,
            enable_header_security: false,
            enable_host_validation: false,
            enable_debug_errors: true,
            enable_cache: false,
            exclusive_login: false,
        }
    }

    /// Configuration for API (minimal)
    pub fn api() -> Self {
        Self {
            enable_csp: false,
            enable_header_security: false,
            enable_host_validation: true,
            enable_debug_errors: true,
            enable_cache: true,
            exclusive_login: false,
        }
    }

    /// Builder pattern for customization
    pub fn custom() -> Self {
        Self::default()
    }

    // Chainable methods for fine-grained configuration
    #[must_use]
    pub fn with_csp(mut self, enable: bool) -> Self {
        self.enable_csp = enable;
        self
    }
    #[must_use]
    pub fn with_debug_errors(mut self, enable: bool) -> Self {
        self.enable_debug_errors = enable;
        self
    }
    #[must_use]
    pub fn with_cache(mut self, enable: bool) -> Self {
        self.enable_cache = enable;
        self
    }
    #[must_use]
    pub fn with_host_validation(mut self, enable: bool) -> Self {
        self.enable_host_validation = enable;
        self
    }
}
