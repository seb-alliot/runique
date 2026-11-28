//! Mailer tracing — email dispatch events.
use tracing::Level;

/// Mailer tracing.
#[derive(Debug, Clone, Default)]
pub struct MailerTracing {
    /// `Email::send()`: backend used, recipient, subject, result (ok/err).
    pub send: Option<Level>,
}

impl MailerTracing {
    /// Creates a config with the mailer channel disabled.
    pub fn new() -> Self {
        Self::default()
    }
    /// Sets the level for `Email::send()` events (backend used, recipient, subject, result).
    #[must_use]
    pub fn send(mut self, level: Level) -> Self {
        self.send = Some(level);
        self
    }
    /// Enables the mailer channel at `Level::DEBUG`.
    pub fn dev(self) -> Self {
        self.send(Level::DEBUG)
    }
}
