//! HTTP error-handling tracing (error_handler_middleware).
use tracing::Level;

/// HTTP error-handling tracing.
#[derive(Debug, Clone, Default)]
pub struct ErrorsTracing {
    /// HTTP error responses caught by the error middleware (status, path).
    pub http: Option<Level>,
    /// Error page rendering (debug diagnostic vs production page).
    pub render: Option<Level>,
}

impl ErrorsTracing {
    /// Creates a config with all error-handling channels disabled.
    pub fn new() -> Self {
        Self::default()
    }
    /// Sets the level for HTTP error responses caught by the error middleware (status, path).
    #[must_use]
    pub fn http(mut self, level: Level) -> Self {
        self.http = Some(level);
        self
    }
    /// Sets the level for error-page rendering (debug diagnostic vs production page).
    #[must_use]
    pub fn render(mut self, level: Level) -> Self {
        self.render = Some(level);
        self
    }
    /// Enables every error-handling channel at `Level::DEBUG`.
    pub fn dev(self) -> Self {
        self.http(Level::DEBUG).render(Level::DEBUG)
    }
}
