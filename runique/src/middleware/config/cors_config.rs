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

    /// The tower-http layer this configuration describes.
    pub(crate) fn layer(&self) -> tower_http::cors::CorsLayer {
        use axum::http::HeaderValue;
        use std::time::Duration;
        use tower_http::cors::{AllowHeaders, AllowMethods, AllowOrigin, Any, CorsLayer};

        let max_age = Duration::from_secs(if self.max_age_secs > 0 {
            self.max_age_secs
        } else {
            3600
        });

        if self.is_wildcard() {
            return CorsLayer::new()
                .allow_origin(Any)
                .allow_methods(Any)
                .allow_headers(Any)
                .max_age(max_age);
        }
        let origins: Vec<HeaderValue> = self
            .origins
            .iter()
            .filter_map(|o| HeaderValue::from_str(o).ok())
            .collect();
        // With credentials, the CORS rules forbid the `*` wildcard (tower-http
        // refuses it at startup): the preflight's own method and headers are
        // echoed back instead, still for the listed origins only.
        let (methods, headers) = if self.allow_credentials {
            (
                AllowMethods::mirror_request(),
                AllowHeaders::mirror_request(),
            )
        } else {
            (AllowMethods::from(Any), AllowHeaders::from(Any))
        };
        CorsLayer::new()
            .allow_origin(AllowOrigin::list(origins))
            .allow_methods(methods)
            .allow_headers(headers)
            .allow_credentials(self.allow_credentials)
            .max_age(max_age)
    }
}
