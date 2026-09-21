//! Auth tracing — session lifecycle + password events.
use tracing::Level;

/// Auth tracing.
#[derive(Debug, Clone, Default)]
pub struct AuthTracing {
    /// User login: session creation, group loading, DB persistence, exclusive flag.
    pub login: Option<Level>,
    /// Password reset flow: token generated, email sent, token validated/consumed, password updated.
    pub reset: Option<Level>,
    /// Warns if `password_init()` is called multiple times.
    pub password_init: Option<Level>,
    /// Permission cache lifecycle: reload from DB after invalidation
    /// (`clear_cache`) — i.e. a request seeing a group's rights change take effect.
    pub permissions: Option<Level>,
}

impl AuthTracing {
    /// Creates a config with all auth channels disabled.
    pub fn new() -> Self {
        Self::default()
    }
    /// Sets the level for login events (session creation, group loading, DB persistence, exclusive flag).
    #[must_use]
    pub fn login(mut self, level: Level) -> Self {
        self.login = Some(level);
        self
    }
    /// Sets the level for password-reset flow events (token generated/sent, validated/consumed, password updated).
    #[must_use]
    pub fn reset(mut self, level: Level) -> Self {
        self.reset = Some(level);
        self
    }
    /// Sets the level for the "`password_init()` called multiple times" warning.
    #[must_use]
    pub fn password_init(mut self, level: Level) -> Self {
        self.password_init = Some(level);
        self
    }
    /// Sets the level for permission-cache reload events (a group's rights change taking effect).
    #[must_use]
    pub fn permissions(mut self, level: Level) -> Self {
        self.permissions = Some(level);
        self
    }
    /// Enables every auth channel at `Level::DEBUG`.
    pub fn dev(self) -> Self {
        self.login(Level::DEBUG)
            .reset(Level::DEBUG)
            .password_init(Level::DEBUG)
            .permissions(Level::DEBUG)
    }
}
