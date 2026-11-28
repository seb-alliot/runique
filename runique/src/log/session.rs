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
    /// Creates a config with all session channels disabled.
    pub fn new() -> Self {
        Self::default()
    }
    /// Sets the level for session-store events (memory watermarks, large records, persistence).
    #[must_use]
    pub fn store(mut self, level: Level) -> Self {
        self.store = Some(level);
        self
    }
    /// Sets the level for the cleanup pass (anonymous sessions purged under memory pressure).
    #[must_use]
    pub fn cleanup(mut self, level: Level) -> Self {
        self.cleanup = Some(level);
        self
    }
    /// Sets the level for other-session invalidation during exclusive login.
    #[must_use]
    pub fn exclusive_login(mut self, level: Level) -> Self {
        self.exclusive_login = Some(level);
        self
    }
    /// Enables every session channel at `Level::DEBUG`.
    pub fn dev(self) -> Self {
        self.store(Level::DEBUG)
            .cleanup(Level::DEBUG)
            .exclusive_login(Level::DEBUG)
    }
}
