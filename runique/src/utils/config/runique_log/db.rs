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
    /// Creates a config with all database channels disabled.
    pub fn new() -> Self {
        Self::default()
    }
    /// Sets the level for connection events (connecting / connected successfully / failure).
    #[must_use]
    pub fn connect(mut self, level: Level) -> Self {
        self.connect = Some(level);
        self
    }
    /// Sets the level for query-level events (slow queries, errors surfaced by the framework).
    #[must_use]
    pub fn query(mut self, level: Level) -> Self {
        self.query = Some(level);
        self
    }
    /// Enables every database channel at `Level::DEBUG`.
    pub fn dev(self) -> Self {
        self.connect(Level::DEBUG).query(Level::DEBUG)
    }
}
