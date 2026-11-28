//! Template engine (Tera) tracing.
use tracing::Level;

/// Template engine (Tera) tracing.
#[derive(Debug, Clone, Default)]
pub struct TemplatesTracing {
    /// Template loading at startup (internal + user templates registered).
    pub load: Option<Level>,
    /// Render-time events (render errors surfaced by the framework).
    pub render: Option<Level>,
}

impl TemplatesTracing {
    /// Creates a config with all template-engine channels disabled.
    pub fn new() -> Self {
        Self::default()
    }
    /// Sets the level for template-loading events at startup (internal + user templates registered).
    #[must_use]
    pub fn load(mut self, level: Level) -> Self {
        self.load = Some(level);
        self
    }
    /// Sets the level for render-time events (render errors surfaced by the framework).
    #[must_use]
    pub fn render(mut self, level: Level) -> Self {
        self.render = Some(level);
        self
    }
    /// Enables every template-engine channel at `Level::DEBUG`.
    pub fn dev(self) -> Self {
        self.load(Level::DEBUG).render(Level::DEBUG)
    }
}
