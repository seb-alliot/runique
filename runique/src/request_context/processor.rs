/// Processor Module - Message Extractor
///
/// Fournit des extracteurs pour simplifier l'accès aux flash messages.
use axum::{extract::FromRequestParts, http::request::Parts, http::StatusCode};
use serde::{Deserialize, Serialize};
use tower_sessions::Session;

/// Clé de session pour les flash messages
pub const FLASH_MESSAGE_KEY: &str = "flash_messages";

/// Représente un flash message dans la session
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FlashMessageData {
    pub level: String, // "success", "error", "info", "warning"
    pub content: String,
}

/// Message Extractor - accès simplifié aux flash messages
///
/// # Exemple
/// ```rust,no_run
/// use axum::http::StatusCode;
/// use runique::request_context::processor::Message;
///
/// async fn my_handler(message: Message) -> &'static str {
///     // Accès aux messages via l'extracteur
///     "OK"
/// }
/// ```
pub struct Message {
    session: Session,
}

impl<S> FromRequestParts<S> for Message
where
    S: Send + Sync,
{
    type Rejection = StatusCode;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let session = Session::from_request_parts(parts, _state)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        Ok(Message { session })
    }
}

impl Message {
    /// Récupère tous les flash messages
    pub async fn get_all(&self) -> Result<Vec<FlashMessageData>, tower_sessions::session::Error> {
        self.session
            .get::<Vec<FlashMessageData>>(FLASH_MESSAGE_KEY)
            .await
            .map(|opt| opt.unwrap_or_default())
    }

    /// Ajoute un flash message
    pub async fn add(
        &self,
        level: impl Into<String>,
        content: impl Into<String>,
    ) -> Result<(), tower_sessions::session::Error> {
        let mut messages = self
            .session
            .get::<Vec<FlashMessageData>>(FLASH_MESSAGE_KEY)
            .await?
            .unwrap_or_default();

        messages.push(FlashMessageData {
            level: level.into(),
            content: content.into(),
        });

        self.session.insert(FLASH_MESSAGE_KEY, messages).await?;

        Ok(())
    }

    /// Ajoute un message de succès
    pub async fn success(
        &self,
        message: impl Into<String>,
    ) -> Result<(), tower_sessions::session::Error> {
        self.add("success", message).await
    }

    /// Ajoute un message d'erreur
    pub async fn error(
        &self,
        message: impl Into<String>,
    ) -> Result<(), tower_sessions::session::Error> {
        self.add("error", message).await
    }

    /// Ajoute un message d'info
    pub async fn info(
        &self,
        message: impl Into<String>,
    ) -> Result<(), tower_sessions::session::Error> {
        self.add("info", message).await
    }

    /// Ajoute un message d'avertissement
    pub async fn warning(
        &self,
        message: impl Into<String>,
    ) -> Result<(), tower_sessions::session::Error> {
        self.add("warning", message).await
    }

    /// Récupère et supprime tous les messages (utilisé dans templates)
    pub async fn pop_all(&self) -> Result<Vec<FlashMessageData>, tower_sessions::session::Error> {
        let messages = self.get_all().await.unwrap_or_default();
        self.session.delete().await?;
        Ok(messages)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_flash_message_data() {
        let msg = FlashMessageData {
            level: "success".to_string(),
            content: "User created".to_string(),
        };
        assert_eq!(msg.level, "success");
        assert_eq!(msg.content, "User created");
    }
}
