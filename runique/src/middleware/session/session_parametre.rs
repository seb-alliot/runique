use std::sync::Arc;
use tower_sessions::{SessionStore, cookie::time::Duration};

pub enum SessionBackend {
    Memory,
    Custom(Arc<dyn SessionStore + Send + Sync>),
}

pub struct SessionConfig {
    pub session: SessionBackend,
    pub duration: Duration,
}

impl SessionConfig {
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
