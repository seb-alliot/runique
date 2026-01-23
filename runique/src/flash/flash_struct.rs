use serde::{Serialize, Deserialize};

pub const FLASH_KEY: &str = "flash_messages";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessageLevel {
    Success,
    Error,
    Info,
    Warning,
}

impl MessageLevel {
    /// Retourne la classe CSS statique associée au niveau du message.
    /// Utilisé dans le rendu html
    pub fn as_css_class(&self) -> &'static str {
        match self {
            MessageLevel::Success => "success-message",
            MessageLevel::Error => "error-message",
            MessageLevel::Info => "info-message",
            MessageLevel::Warning => "warning-message",
        }
    }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlashMessage {
    pub content: String,
    pub level: MessageLevel,
}

impl FlashMessage {
    /// Crée un nouveau message flash.
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



