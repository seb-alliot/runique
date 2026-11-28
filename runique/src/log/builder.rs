//! Builder startup tracing — one-time events during `build()`.
use tracing::Level;

/// Builder startup tracing — one-time events during `build()`.
#[derive(Debug, Clone, Default)]
pub struct BuilderTracing {
    /// Template loading: nb internal + user templates registered in Tera.
    pub templates: Option<Level>,
    /// Admin registry: nb resources registered at startup.
    pub registry: Option<Level>,
    /// Middleware stack: each slot name + number assigned at startup.
    pub middleware: Option<Level>,
    /// Static files: static_url + path + media_url + path.
    pub statics: Option<Level>,
    /// URL routes: nb named routes in registry after `add_urls()`.
    pub routes: Option<Level>,
}

impl BuilderTracing {
    /// Creates a config with all builder startup channels disabled.
    pub fn new() -> Self {
        Self::default()
    }
    /// Sets the level for template-loading events (nb internal + user templates registered).
    #[must_use]
    pub fn templates(mut self, level: Level) -> Self {
        self.templates = Some(level);
        self
    }
    /// Sets the level for admin registry events (nb resources registered at startup).
    #[must_use]
    pub fn registry(mut self, level: Level) -> Self {
        self.registry = Some(level);
        self
    }
    /// Sets the level for middleware stack events (each slot name + number assigned at startup).
    #[must_use]
    pub fn middleware(mut self, level: Level) -> Self {
        self.middleware = Some(level);
        self
    }
    /// Sets the level for static-files setup events (static_url + path + media_url + path).
    #[must_use]
    pub fn statics(mut self, level: Level) -> Self {
        self.statics = Some(level);
        self
    }
    /// Sets the level for URL route registration events (nb named routes after `add_urls()`).
    #[must_use]
    pub fn routes(mut self, level: Level) -> Self {
        self.routes = Some(level);
        self
    }
    /// Enables every builder startup channel at `Level::DEBUG`.
    pub fn dev(self) -> Self {
        self.templates(Level::DEBUG)
            .registry(Level::DEBUG)
            .middleware(Level::DEBUG)
            .statics(Level::DEBUG)
            .routes(Level::DEBUG)
    }
}
