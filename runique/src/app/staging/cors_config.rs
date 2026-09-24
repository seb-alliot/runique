/// CORS configuration passed via closure to `.with_cors(|c| { ... })`.
///
/// Disabled by default — explicitly configure origins.
/// Validation at build time: wildcard origin + `allow_credentials(true)` = BuildError.
///
/// # Example — separate frontend
/// ```rust,ignore
/// .middleware(|m| {
///     m.with_cors(|c| {
///         c.origin("https://app.mysite.com")
///          .allow_credentials(true)
///     })
/// })
/// ```
///
/// # Example — public API without a session
/// ```rust,ignore
/// m.with_cors(|c| c.any_origin())
/// ```
#[derive(Default)]
pub struct CorsConfig {
    pub(crate) origins: Vec<String>,
    pub(crate) allow_credentials: bool,
    pub(crate) max_age_secs: u64,
}

impl CorsConfig {
    /// Allows one specific origin (can be called several times).
    pub fn origin(mut self, origin: impl Into<String>) -> Self {
        self.origins.push(origin.into());
        self
    }

    /// Allows every origin (`*`). Incompatible with `allow_credentials(true)`.
    pub fn any_origin(mut self) -> Self {
        self.origins = vec!["*".to_string()];
        self
    }

    /// Allows cross-origin cookies and auth headers.
    /// Forbidden if `any_origin()` is configured — BuildError at startup.
    pub fn allow_credentials(mut self, allow: bool) -> Self {
        self.allow_credentials = allow;
        self
    }

    /// How long preflight responses are cached (seconds, default: 3600).
    pub fn max_age(mut self, secs: u64) -> Self {
        self.max_age_secs = secs;
        self
    }

    pub(crate) fn is_wildcard(&self) -> bool {
        self.origins.iter().any(|o| o == "*")
    }
}
