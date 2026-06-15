//! Database tracing — connection + query events.
use tracing::Level;

/// Database tracing.
#[derive(Debug, Clone, Default)]
pub struct DbTracing {
    /// Connection info (connecting / connected successfully / failure).
    pub connect: Option<Level>,
    /// Query-level events (slow queries, errors surfaced by the framework).
    pub query: Option<Level>,
}

impl DbTracing {
    pub fn new() -> Self {
        Self::default()
    }
    #[must_use]
    pub fn connect(mut self, level: Level) -> Self {
        self.connect = Some(level);
        self
    }
    #[must_use]
    pub fn query(mut self, level: Level) -> Self {
        self.query = Some(level);
        self
    }
    pub fn dev(self) -> Self {
        self.connect(Level::DEBUG).query(Level::DEBUG)
    }
}
