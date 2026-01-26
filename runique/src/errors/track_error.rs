use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use thiserror::Error;
use tracing::{error, info};

/// Type Result global pour Runique
pub type RuniqueResult<T> = Result<T, RuniqueError>;

/// Erreurs applicatives centralisées
#[derive(Debug, Error)]
pub enum RuniqueError {
    #[error("Erreur interne")]
    Internal,

    #[error("Accès interdit")]
    Forbidden,

    #[error("Ressource introuvable")]
    NotFound,

    #[error("Erreur de validation: {0}")]
    Validation(String),

    #[error("Erreur base de données: {0}")]
    Database(String),

    #[error("Erreur IO: {0}")]
    Io(String), // <- Stocke juste le message de l'erreur

    #[error("Erreur template: {0}")]
    Template(String),

    #[error("{message}")]
    Custom {
        message: String,
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },
}

impl Clone for RuniqueError {
    fn clone(&self) -> Self {
        match self {
            RuniqueError::Internal => RuniqueError::Internal,
            RuniqueError::Forbidden => RuniqueError::Forbidden,
            RuniqueError::NotFound => RuniqueError::NotFound,
            RuniqueError::Validation(msg) => RuniqueError::Validation(msg.clone()),
            RuniqueError::Database(msg) => RuniqueError::Database(msg.clone()),
            RuniqueError::Io(msg) => RuniqueError::Io(msg.clone()),
            RuniqueError::Template(msg) => RuniqueError::Template(msg.clone()),
            RuniqueError::Custom { message, source: _ } => RuniqueError::Custom {
                message: message.clone(),
                source: None, // Cannot clone the source error
            },
        }
    }
}

impl From<std::io::Error> for RuniqueError {
    fn from(err: std::io::Error) -> Self {
        RuniqueError::Io(err.to_string())
    }
}

impl RuniqueError {
    /// Log the error using tracing
    pub fn log(&self) {
        match self {
            RuniqueError::Internal => error!("Erreur interne"),
            RuniqueError::Forbidden => info!("Accès interdit"),
            RuniqueError::NotFound => info!("Ressource introuvable"),
            RuniqueError::Validation(msg) => info!("Erreur de validation: {}", msg),
            RuniqueError::Database(msg) => error!("Erreur base de données: {}", msg),
            RuniqueError::Io(msg) => error!("Erreur IO: {}", msg),
            RuniqueError::Template(msg) => error!("Erreur template: {}", msg),
            RuniqueError::Custom { message, source } => {
                error!("Erreur custom: {}", message);
                if let Some(source) = source.as_ref() {
                    error!("Source: {}", source);
                }
            }
        }
    }
}

/// Conversion en réponse HTTP (Axum)
impl IntoResponse for RuniqueError {
    fn into_response(self) -> Response {
        // Log automatiquement
        self.log();

        let (status, message) = match &self {
            RuniqueError::NotFound => (StatusCode::NOT_FOUND, self.to_string()),
            RuniqueError::Forbidden => (StatusCode::FORBIDDEN, self.to_string()),
            RuniqueError::Validation(_) => (StatusCode::BAD_REQUEST, self.to_string()),
            _ => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Erreur interne".to_string(),
            ),
        };

        (status, message).into_response()
    }
}
