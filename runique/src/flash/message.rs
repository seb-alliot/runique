//! Flash message structs — `FlashMessage` and `MessageLevel` with CSS mapping.
use serde::{Deserialize, Serialize};

/// Serialized in lowercase: `message.html` builds the CSS class from it
/// (`message-{{ message.level }}` → `message-success`).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MessageLevel {
    Success,
    Error,
    Info,
    Warning,
}

impl MessageLevel {
    /// Returns the CSS class `message.html` gives this level.
    pub fn as_css_class(&self) -> &'static str {
        match self {
            MessageLevel::Success => "message-success",
            MessageLevel::Error => "message-error",
            MessageLevel::Info => "message-info",
            MessageLevel::Warning => "message-warning",
        }
    }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlashMessage {
    pub content: String,
    pub level: MessageLevel,
}

impl FlashMessage {
    /// Creates a new flash message.
    pub fn new<S: Into<String>>(content: S, level: MessageLevel) -> Self {
        FlashMessage {
            content: content.into(),
            level,
        }
    }
    pub fn success<S: Into<String>>(content: S) -> Self {
        FlashMessage {
            content: content.into(),
            level: MessageLevel::Success,
        }
    }
    pub fn error<S: Into<String>>(content: S) -> Self {
        FlashMessage {
            content: content.into(),
            level: MessageLevel::Error,
        }
    }
    pub fn info<S: Into<String>>(content: S) -> Self {
        FlashMessage {
            content: content.into(),
            level: MessageLevel::Info,
        }
    }
    pub fn warning<S: Into<String>>(content: S) -> Self {
        FlashMessage {
            content: content.into(),
            level: MessageLevel::Warning,
        }
    }
}
