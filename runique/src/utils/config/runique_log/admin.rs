//! Admin panel tracing — per-operation granularity.
use tracing::Level;

/// Admin panel tracing.
#[derive(Debug, Clone, Default)]
pub struct AdminTracing {
    /// Auth checks: login, permission gate, write-access guard.
    pub auth: Option<Level>,
    /// CRUD handlers: detail, create, edit, delete — request + outcome.
    pub crud: Option<Level>,
    /// List view: pagination, ordering, column resolution.
    pub list: Option<Level>,
    /// Bulk operations: group_action, group_set, bulk_delete.
    pub bulk: Option<Level>,
    /// `filter_fn` failures in the admin list view.
    pub filter_fn: Option<Level>,
    /// Admin roles registry access errors.
    pub roles: Option<Level>,
    /// Admin daemon / generated resource events.
    pub daemon: Option<Level>,
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
    /// Sets the level for `filter_fn` failures in the admin list view.
    #[must_use]
    pub fn filter_fn(mut self, level: Level) -> Self {
        self.filter_fn = Some(level);
        self
    }
    /// Sets the level for admin roles registry access errors.
    #[must_use]
    pub fn roles(mut self, level: Level) -> Self {
        self.roles = Some(level);
        self
    }
    /// Sets the level for admin daemon / generated resource events.
    #[must_use]
    pub fn daemon(mut self, level: Level) -> Self {
        self.daemon = Some(level);
        self
    }
    /// Enables every admin panel channel at `Level::DEBUG`.
    pub fn dev(self) -> Self {
        self.auth(Level::DEBUG)
            .crud(Level::DEBUG)
            .list(Level::DEBUG)
            .bulk(Level::DEBUG)
            .filter_fn(Level::DEBUG)
            .roles(Level::DEBUG)
            .daemon(Level::DEBUG)
    }
}
