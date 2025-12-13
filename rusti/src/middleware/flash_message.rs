use serde::{Serialize, Deserialize};
use async_trait::async_trait;
use tower_sessions::{Session};
use tower_sessions::session::Error as SessionError;
use axum::{
    response::IntoResponse,
    response::Response,
    http::StatusCode,
    middleware::Next,
    body::Body,
};

/// Représente le type d'un message flash.
/// Utilisé pour déterminer le style d'affichage.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessageLevel {
    Success,
    Error,
    Info,
}

impl MessageLevel {
    /// Retourne la classe CSS statique associée au niveau du message.
    pub fn as_css_class(&self) -> &'static str {
        match self {
            MessageLevel::Success => "success-message",
            MessageLevel::Error => "error-message",
            MessageLevel::Info => "info-message",
        }
    }

    pub fn success() -> Self {
        MessageLevel::Success
    }
    pub fn error() -> Self {
        MessageLevel::Error
    }
    pub fn info() -> Self {
        MessageLevel::Info
    }
}

/// Structure représentant un message flash complet, incluant le contenu et le niveau.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    /// Le contenu textuel du message.
    pub content: String,
    /// Le niveau de gravité du message (Success, Error, Info).
    pub level: MessageLevel,
}

impl Message {
    /// Crée un nouveau message flash.
    pub fn new<S: Into<String>>(content: S, level: MessageLevel) -> Self {
        Message {
            content: content.into(),
            level,
        }
    }

    pub fn success<S: Into<String>>(content: S) -> Self {
        Message {
            content: content.into(),
            level: MessageLevel::Success,
        }
    }

    pub fn error<S: Into<String>>(content: S) -> Self {
        Message {
            content: content.into(),
            level: MessageLevel::Error,
        }
    }

    pub fn info<S: Into<String>>(content: S) -> Self {
        Message {
            content: content.into(),
            level: MessageLevel::Info,
        }
    }
}

/// Clé utilisée pour stocker le vecteur de messages dans la session Tower.
const FLASH_MESSAGES_KEY: &str = "flash_messages";

/// Trait d'extension pour simplifier l'interaction avec les messages flash
/// sur l'objet `tower_sessions::Session`.

#[async_trait]
pub trait FlashMessageSession {
    /// Ajoute un message à la liste des messages flash stockés dans la session.
    ///
    /// // session.insert_flash_message(Message::success("Opération réussie")).await?;
    /// ```
    async fn insert_message(&mut self, message: Message) -> Result<(), SessionError>;

    /// Lit (consomme) tous les messages flash stockés et les retire immédiatement de la session.
    ///
    /// # Remarques
    /// C'est cette méthode qui assure que les messages sont affichés une seule fois.
    ///
    /// # Exemple
    /// ```
    /// // let messages = session.remove_messages().await.unwrap_or_default();
    /// ```
    async fn remove_messages(&mut self) -> Result<Vec<Message>, SessionError>;
}

#[async_trait]
impl FlashMessageSession for Session {

    async fn insert_message(&mut self, message: Message) -> Result<(), SessionError> {
        // Tenter de récupérer la liste actuelle des messages (Vec<Message>)
        // Le turbofish <Vec<Message>> est nécessaire pour la désérialisation
        let mut messages: Vec<Message> = self.get::<Vec<Message>>(FLASH_MESSAGES_KEY)
            .await?
            .unwrap_or_default(); // Crée une nouvelle liste si la clé n'existe pas

        messages.push(message);

        // Enregistrer la liste mise à jour
        self.insert(FLASH_MESSAGES_KEY, messages).await
    }

    // Lire et supprimer tous les messages flash de la session.
    async fn remove_messages(&mut self) -> Result<Vec<Message>, SessionError> {
        // 1. Lire les messages depuis la session.
        // Le turbofish <Vec<Message>> est nécessaire pour la désérialisation
        let messages: Vec<Message> = self.get::<Vec<Message>>(FLASH_MESSAGES_KEY)
            .await?
            .unwrap_or_default();

        // 2. Supprimer la clé de session si des messages ont été lus.
        // Permet la lecture une seule fois avant d'être supprimés.
        if !messages.is_empty() {
            self.remove::<Vec<Message>>(FLASH_MESSAGES_KEY).await?;
        }

        // 3. Retourner les messages lus.
        Ok(messages)
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

/// Ajoute un message de succès à la session flash.
/// Utilisé dans les handlers Axum.
pub async fn flash_success<S: Into<String>>(
    session: &mut Session,
    content: S,
) -> Result<(), SessionError> {
    session.insert_message(Message::success(content.into())).await
}

pub async fn flash_error<S: Into<String>>(
    session: &mut Session,
    content: S,
) -> Result<(), SessionError> {
    session.insert_message(Message::error(content.into())).await
}

pub async fn flash_info<S: Into<String>>(
    session: &mut Session,
    content: S,
) -> Result<(), SessionError> {
    session.insert_message(Message::info(content.into())).await
}


/// Middleware Axum pour gérer les messages flash.
/// Lit les messages flash de la session et les insère dans les extensions de la requête
/// les supprime ensuite de la session pour assurer qu'ils ne sont lus qu'une seule fois.
pub async fn flash_middleware(
    mut req: axum::http::Request<Body>,
    next: Next,
) -> Response {

    // Étape 1 : extraire les messages sans toucher aux extensions ensuite
    let messages = {
        let session = match req.extensions_mut().get_mut::<Session>() {
            Some(s) => s,
            None => return next.run(req).await,
        };
        

        let messages = session
        .get::<Vec<Message>>(FLASH_MESSAGES_KEY)
        .await
        .ok()
        .flatten()
        .unwrap_or_default();

        if !messages.is_empty() {
            // Supprimer les messages après les avoir lus
            let _ = session.remove::<Vec<Message>>(FLASH_MESSAGES_KEY).await;
        }
        messages
    };
    // Étape 2 : insérer les messages dans les extensions pour le traitement ultérieur
    if !messages.is_empty() {
        req.extensions_mut().insert(messages);
    }
    next.run(req).await
}
