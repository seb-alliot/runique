/// CORS configuration passed via closure to `.with_cors(|c| { ... })`.
///
/// Disabled by default — explicitly configure origins.
/// Validation at build time: wildcard origin + `allow_credentials(true)` = BuildError.
///
/// # Example — frontend séparé
/// ```rust,ignore
/// .middleware(|m| {
///     m.with_cors(|c| {
///         c.origin("https://app.monsite.com")
///          .allow_credentials(true)
///     })
/// })
/// ```
///
/// # Example — API publique sans session
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
    /// Autorise une origine spécifique (appelable plusieurs fois).
    pub fn origin(mut self, origin: impl Into<String>) -> Self {
        self.origins.push(origin.into());
        self
    }

    /// Autorise toutes les origines (`*`). Incompatible avec `allow_credentials(true)`.
    pub fn any_origin(mut self) -> Self {
        self.origins = vec!["*".to_string()];
        self
    }

    /// Autorise les cookies et headers d'auth cross-origin.
    /// Interdit si `any_origin()` est configuré — BuildError au démarrage.
    pub fn allow_credentials(mut self, allow: bool) -> Self {
        self.allow_credentials = allow;
        self
    }

    /// Durée de mise en cache des réponses preflight (secondes, défaut: 3600).
    pub fn max_age(mut self, secs: u64) -> Self {
        self.max_age_secs = secs;
        self
    }

    pub(crate) fn is_wildcard(&self) -> bool {
        self.origins.iter().any(|o| o == "*")
    }
}
