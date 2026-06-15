//! Migration / makemigrations CLI tracing.
use tracing::Level;

/// Migration / makemigrations tracing.
#[derive(Debug, Clone, Default)]
pub struct MigrationTracing {
    /// Plan phase: diff computed, statements generated.
    pub plan: Option<Level>,
    /// Apply phase: statements executed, snapshot committed.
    pub apply: Option<Level>,
    /// Rollback phase: atomic revert on failure.
    pub rollback: Option<Level>,
}

impl MigrationTracing {
    pub fn new() -> Self {
        Self::default()
    }
    #[must_use]
    pub fn plan(mut self, level: Level) -> Self {
        self.plan = Some(level);
        self
    }
    #[must_use]
    pub fn apply(mut self, level: Level) -> Self {
        self.apply = Some(level);
        self
    }
    #[must_use]
    pub fn rollback(mut self, level: Level) -> Self {
        self.rollback = Some(level);
        self
    }
    pub fn dev(self) -> Self {
        self.plan(Level::DEBUG)
            .apply(Level::DEBUG)
            .rollback(Level::DEBUG)
    }
}
