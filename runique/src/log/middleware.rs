//! Middleware/security tracing — one field per security middleware.
use tracing::Level;

/// Middleware/security tracing.
#[derive(Debug, Clone, Default)]
pub struct MiddlewareTracing {
    /// Detects a `csrf_token` in a GET URL (silent cleanup) and CSRF rejections.
    pub csrf: Option<Level>,
    /// CSP policy build + nonce injection events.
    pub csp: Option<Level>,
    /// CORS middleware decisions (when enabled).
    pub cors: Option<Level>,
    /// Rate limiter: requests blocked (ip, retry_after).
    pub rate_limit: Option<Level>,
    /// Host header validation rejections (HTTP/2 `:authority` fallback included).
    pub host_validation: Option<Level>,
    /// Open redirect: external redirect attempts blocked.
    pub open_redirect: Option<Level>,
    /// Anti-bot honeypot triggers.
    pub anti_bot: Option<Level>,
    /// HTTPS/ACME-TLS lifecycle: cert loaded, renewed, binding port 443, HTTP→HTTPS upgrade.
    pub https: Option<Level>,
}

impl MiddlewareTracing {
    /// Creates a config with all middleware channels disabled.
    pub fn new() -> Self {
        Self::default()
    }
    /// Sets the level for CSRF-related events (GET-URL token cleanup, rejections).
    #[must_use]
    pub fn csrf(mut self, level: Level) -> Self {
        self.csrf = Some(level);
        self
    }
    /// Sets the level for CSP policy build and nonce injection events.
    #[must_use]
    pub fn csp(mut self, level: Level) -> Self {
        self.csp = Some(level);
        self
    }
    /// Sets the level for CORS middleware decisions.
    #[must_use]
    pub fn cors(mut self, level: Level) -> Self {
        self.cors = Some(level);
        self
    }
    /// Sets the level for rate-limiter block events.
    #[must_use]
    pub fn rate_limit(mut self, level: Level) -> Self {
        self.rate_limit = Some(level);
        self
    }
    /// Sets the level for Host-header validation rejections.
    #[must_use]
    pub fn host_validation(mut self, level: Level) -> Self {
        self.host_validation = Some(level);
        self
    }
    /// Sets the level for blocked open-redirect attempts.
    #[must_use]
    pub fn open_redirect(mut self, level: Level) -> Self {
        self.open_redirect = Some(level);
        self
    }
    /// Sets the level for anti-bot honeypot triggers.
    #[must_use]
    pub fn anti_bot(mut self, level: Level) -> Self {
        self.anti_bot = Some(level);
        self
    }
    /// Sets the level for HTTPS/ACME-TLS lifecycle events.
    #[must_use]
    pub fn https(mut self, level: Level) -> Self {
        self.https = Some(level);
        self
    }
    /// Enables every middleware channel at `Level::DEBUG`.
    pub fn dev(self) -> Self {
        self.csrf(Level::DEBUG)
            .csp(Level::DEBUG)
            .cors(Level::DEBUG)
            .rate_limit(Level::DEBUG)
            .host_validation(Level::DEBUG)
            .open_redirect(Level::DEBUG)
            .anti_bot(Level::DEBUG)
            .https(Level::DEBUG)
    }
}
