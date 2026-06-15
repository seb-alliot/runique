//! Mailer tracing — email dispatch events.
use tracing::Level;

/// Mailer tracing.
#[derive(Debug, Clone, Default)]
pub struct MailerTracing {
    /// `Email::send()`: backend used, recipient, subject, result (ok/err).
    pub send: Option<Level>,
}

impl MailerTracing {
    pub fn new() -> Self {
        Self::default()
    }
    #[must_use]
    pub fn send(mut self, level: Level) -> Self {
        self.send = Some(level);
        self
    }
    pub fn dev(self) -> Self {
        self.send(Level::DEBUG)
    }
}
