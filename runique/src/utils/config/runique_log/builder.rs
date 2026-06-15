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
    pub fn new() -> Self {
        Self::default()
    }
    #[must_use]
    pub fn templates(mut self, level: Level) -> Self {
        self.templates = Some(level);
        self
    }
    #[must_use]
    pub fn registry(mut self, level: Level) -> Self {
        self.registry = Some(level);
        self
    }
    #[must_use]
    pub fn middleware(mut self, level: Level) -> Self {
        self.middleware = Some(level);
        self
    }
    #[must_use]
    pub fn statics(mut self, level: Level) -> Self {
        self.statics = Some(level);
        self
    }
    #[must_use]
    pub fn routes(mut self, level: Level) -> Self {
        self.routes = Some(level);
        self
    }
    pub fn dev(self) -> Self {
        self.templates(Level::DEBUG)
            .registry(Level::DEBUG)
            .middleware(Level::DEBUG)
            .statics(Level::DEBUG)
            .routes(Level::DEBUG)
    }
}
