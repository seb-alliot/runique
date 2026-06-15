//! Session lifecycle tracing.
use tracing::Level;

/// Session lifecycle tracing.
#[derive(Debug, Clone, Default)]
pub struct SessionTracing {
    /// Session store traces: memory watermarks, large records, persistence.
    pub store: Option<Level>,
    /// Cleanup pass: anonymous sessions purged under memory pressure.
    pub cleanup: Option<Level>,
    /// Other-session invalidation during exclusive login.
    pub exclusive_login: Option<Level>,
}

impl SessionTracing {
    pub fn new() -> Self {
        Self::default()
    }
    #[must_use]
    pub fn store(mut self, level: Level) -> Self {
        self.store = Some(level);
        self
    }
    #[must_use]
    pub fn cleanup(mut self, level: Level) -> Self {
        self.cleanup = Some(level);
        self
    }
    #[must_use]
    pub fn exclusive_login(mut self, level: Level) -> Self {
        self.exclusive_login = Some(level);
        self
    }
    pub fn dev(self) -> Self {
        self.store(Level::DEBUG)
            .cleanup(Level::DEBUG)
            .exclusive_login(Level::DEBUG)
    }
}
