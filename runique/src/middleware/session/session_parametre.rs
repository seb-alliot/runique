//! Session parameters — `SessionBackend` (Memory/Custom), memory limits, lifetime.
use std::sync::Arc;
use tower_sessions::{SessionStore, cookie::time::Duration};

/// Selects the `tower_sessions::SessionStore` backend used for anonymous and
/// CSRF sessions (authenticated sessions are always persisted through
/// `RuniqueSessionStore` separately).
pub enum SessionBackend {
    /// In-memory store — the default; state is lost on restart.
    Memory,
    /// A caller-provided `SessionStore` implementation (e.g. Redis-backed).
    Custom(Arc<dyn SessionStore + Send + Sync>),
}

/// Session middleware configuration: backend and cookie lifetime.
pub struct SessionConfig {
    /// Store backend used for session persistence.
    pub session: SessionBackend,
    /// Session lifetime before expiry. Default: 24 hours.
    pub duration: Duration,
}

impl SessionConfig {
    /// Overrides the session lifetime (default: 24 hours).
    #[must_use]
    pub fn with_duration(mut self, duration: Duration) -> Self {
        self.duration = duration;
        self
    }
}

impl Default for SessionConfig {
    fn default() -> Self {
        Self {
            session: SessionBackend::Memory,
            duration: Duration::seconds(86400),
        }
    }
}
