use axum::http::{request::Parts, StatusCode};
use tower_sessions::Session;

// Import depuis ton nouveau dossier de middleware
use crate::gardefou::composant_middleware::flash_message::{FlashMessage, FlashMessageSession};

#[derive(Clone)]
pub struct FlashManager(pub Session);

impl FlashManager {
    /// Permet à l'extracteur RuniqueContext de créer le manager
    pub fn from_parts(parts: &Parts) -> Result<Self, StatusCode> {
        let session = parts
            .extensions
            .get::<Session>()
            .ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;
        Ok(Self(session.clone()))
    }

    pub async fn success<S: Into<String>>(&mut self, content: S) -> Result<(), StatusCode> {
        self.0
            .insert_message(FlashMessage::success(content.into()))
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
    }

    pub async fn error<S: Into<String>>(&mut self, content: S) -> Result<(), StatusCode> {
        self.0
            .insert_message(FlashMessage::error(content.into()))
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
    }

    pub async fn info<S: Into<String>>(&mut self, content: S) -> Result<(), StatusCode> {
        self.0
            .insert_message(FlashMessage::info(content.into()))
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
    }

    pub async fn warning<S: Into<String>>(&mut self, content: S) -> Result<(), StatusCode> {
        self.0
            .insert_message(FlashMessage::warning(content.into()))
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
    }
}
