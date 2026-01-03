use async_trait::async_trait;
use axum::{
    body::Body, http::StatusCode, middleware::Next, response::IntoResponse, response::Response,
};
use serde::{Deserialize, Serialize};
use tower_sessions::Session;
use tower_sessions::session::Error as SessionError;

/// Représente le type d'un message flash.
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

/// Structure représentant un message flash complet, incluant le contenu et le niveau.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlashMessage {
    /// Le contenu textuel du message.
    pub content: String,
    /// Le niveau de gravité du message (Success, Error, Info).
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

/// Clé utilisée pour stocker le vecteur de messages dans la session Tower.
const FLASH_MESSAGES_KEY: &str = "flash_messages";

/// sur l'objet `tower_sessions::Session`.

#[async_trait]
pub trait FlashMessageSession {
    /// Ajoute un message à la liste des messages flash stockés dans la session.
    /// Exemple d'utilisation :
    /// // session.insert_flash_message(FlashMessage::success("Opération réussie")).await?;
    async fn insert_message(&mut self, message: FlashMessage) -> Result<(), SessionError>;
}

#[async_trait]
impl FlashMessageSession for Session {
    async fn insert_message(&mut self, message: FlashMessage) -> Result<(), SessionError> {
        // Tenter de récupérer la liste actuelle des messages (Vec<Message>)
        // Le turbofish <Vec<Message>> est nécessaire pour la désérialisation
        let mut messages: Vec<FlashMessage> = self
            .get::<Vec<FlashMessage>>(FLASH_MESSAGES_KEY)
            .await?
            .unwrap_or_default();
        messages.push(message);
        // Enregistrer la liste mise à jour
        self.insert(FLASH_MESSAGES_KEY, messages).await
    }
}

/// Erreur de rejet pour l'Extractor. Nécessite IntoResponse pour qu'Axum la gère.
pub struct FlashContextError(SessionError);

impl IntoResponse for FlashContextError {
    fn into_response(self) -> Response {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Erreur lors de la lecture des messages flash: {}", self.0),
        )
            .into_response()
    }
}

/// Middleware Axum pour gérer les messages flash.
/// Lit les messages flash de la session et les insère dans les extensions de la requête
/// les supprime ensuite de la session pour assurer qu'ils ne sont lus qu'une seule fois.
pub async fn flash_middleware(mut req: axum::http::Request<Body>, next: Next) -> Response {
    // Étape 1 : extraire les messages sans toucher aux extensions ensuite
    let messages: Vec<FlashMessage> = {
        let session = match req.extensions_mut().get_mut::<Session>() {
            Some(s) => s,
            None => return next.run(req).await,
        };

        let messages = session
            .get::<Vec<FlashMessage>>(FLASH_MESSAGES_KEY)
            .await
            .ok()
            .flatten()
            .unwrap_or_default();

        if !messages.is_empty() {
            // Supprimer les messages après les avoir lus
            let _ = session
                .remove::<Vec<FlashMessage>>(FLASH_MESSAGES_KEY)
                .await;
        }
        messages
    };
    // Étape 2 : insérer les messages dans les extensions pour le traitement ultérieur
    if !messages.is_empty() {
        req.extensions_mut().insert(messages);
    }
    next.run(req).await
}
