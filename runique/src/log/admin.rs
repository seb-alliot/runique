//! Admin panel tracing — per-operation granularity.
use tracing::Level;

/// Admin panel tracing.
#[derive(Debug, Clone, Default)]
pub struct AdminTracing {
    /// Auth checks: login, permission gate, write-access guard.
    pub auth: Option<Level>,
    /// Writes through the admin's forms: create, edit, delete — done or refused.
    pub crud: Option<Level>,
    /// List view: page, rows, total, filters and search applied; dashboard counts.
    pub list: Option<Level>,
    /// Bulk actions: done (with the number of rows) or refused.
    pub bulk: Option<Level>,
}

impl AdminTracing {
    /// Creates a config with all admin panel channels disabled.
    pub fn new() -> Self {
        Self::default()
    }
    /// Sets the level for admin auth checks (login, permission gate, write-access guard).
    #[must_use]
    pub fn auth(mut self, level: Level) -> Self {
        self.auth = Some(level);
        self
    }
    /// Sets the level for CRUD handler events (detail, create, edit, delete).
    #[must_use]
    pub fn crud(mut self, level: Level) -> Self {
        self.crud = Some(level);
        self
    }
    /// Sets the level for list-view events (pagination, ordering, column resolution).
    #[must_use]
    pub fn list(mut self, level: Level) -> Self {
        self.list = Some(level);
        self
    }
    /// Sets the level for bulk operations (`group_action`, `group_set`, `bulk_delete`).
    #[must_use]
    pub fn bulk(mut self, level: Level) -> Self {
        self.bulk = Some(level);
        self
    }
    /// Enables every admin panel channel at `Level::DEBUG`.
    pub fn dev(self) -> Self {
        self.auth(Level::DEBUG)
            .crud(Level::DEBUG)
            .list(Level::DEBUG)
            .bulk(Level::DEBUG)
    }
}
