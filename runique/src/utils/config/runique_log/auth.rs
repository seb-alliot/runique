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
}

impl AuthTracing {
    pub fn new() -> Self {
        Self::default()
    }
    #[must_use]
    pub fn login(mut self, level: Level) -> Self {
        self.login = Some(level);
        self
    }
    #[must_use]
    pub fn reset(mut self, level: Level) -> Self {
        self.reset = Some(level);
        self
    }
    #[must_use]
    pub fn password_init(mut self, level: Level) -> Self {
        self.password_init = Some(level);
        self
    }
    pub fn dev(self) -> Self {
        self.login(Level::DEBUG)
            .reset(Level::DEBUG)
            .password_init(Level::DEBUG)
    }
}
