//! `RuniqueError`: what went wrong inside the framework, by kind, with the
//! level it is logged at and the response it becomes.
use super::build::BuildError;
use super::context::{ErrorContext, ErrorType};
use crate::translation::{t, tf};
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use std::sync::Arc;
use thiserror::Error;
use tracing::{error, info};

/// Global `Result` alias for the framework.
pub type RuniqueResult<T> = Result<T, RuniqueError>;

/// Centralized application errors of the framework.
#[derive(Debug, Error)]
pub enum RuniqueError {
    #[error("Build error: {0}")]
    Build(BuildError),
    #[error("Internal error")]
    Internal,
    #[error("Access denied")]
    Forbidden,
    #[error("Resource not found")]
    NotFound,
    #[error("Validation error: {0}")]
    Validation(String),
    #[error("Database error: {0}")]
    Database(String),
    #[error("IO error: {0}")]
    Io(String),
    #[error("Template error: {message}")]
    Template {
        message: String,
        /// 1-based line of the failing template source, read from the Tera
        /// error's span (`None` when the error carries no span).
        line: Option<usize>,
    },
    #[error("{message}")]
    Custom {
        message: String,
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },
}

impl Clone for RuniqueError {
    fn clone(&self) -> Self {
        match self {
            RuniqueError::Build(e) => RuniqueError::Build(e.clone()),
            RuniqueError::Internal => RuniqueError::Internal,
            RuniqueError::Forbidden => RuniqueError::Forbidden,
            RuniqueError::NotFound => RuniqueError::NotFound,
            RuniqueError::Validation(msg) => RuniqueError::Validation(msg.clone()),
            RuniqueError::Database(msg) => RuniqueError::Database(msg.clone()),
            RuniqueError::Io(msg) => RuniqueError::Io(msg.clone()),
            RuniqueError::Template { message, line } => RuniqueError::Template {
                message: message.clone(),
                line: *line,
            },
            RuniqueError::Custom { message, source: _ } => RuniqueError::Custom {
                message: message.clone(),
                source: None,
            },
        }
    }
}

impl From<std::io::Error> for RuniqueError {
    fn from(err: std::io::Error) -> Self {
        RuniqueError::Io(err.to_string())
    }
}

/// The 1-based line a Tera error points at, read from its span. Tera 2 no longer
/// puts a line in the `Display` text (it emits a codespan report), so the span
/// on the error kind is the only reliable source. `#[non_exhaustive]` on
/// `ErrorKind` forces the catch-all arm.
#[must_use]
pub fn tera_error_line(error: &tera::Error) -> Option<usize> {
    match error.kind() {
        tera::ErrorKind::RenderingError(report) | tera::ErrorKind::SyntaxError(report) => {
            Some(report.span().start_line)
        }
        _ => None,
    }
}

impl RuniqueError {
    /// Builds a `Template` error from a Tera error, preserving the failing line.
    #[must_use]
    pub fn template(error: &tera::Error) -> Self {
        RuniqueError::Template {
            message: error.to_string(),
            line: tera_error_line(error),
        }
    }
}

impl From<BuildError> for RuniqueError {
    fn from(err: BuildError) -> Self {
        RuniqueError::Build(err)
    }
}
impl RuniqueError {
    /// Logs the error with the appropriate tracing level (error/info).
    pub fn log(&self) {
        match self {
            RuniqueError::Build(e) => error!(
                "{}",
                tf(runique_i18n::keys::app::error::BUILD, &[&e.to_string()])
            ),
            RuniqueError::Internal => error!("{}", t(runique_i18n::keys::app::error::INTERNAL)),
            RuniqueError::Forbidden => info!("{}", t(runique_i18n::keys::app::error::FORBIDDEN)),
            RuniqueError::NotFound => info!("{}", t(runique_i18n::keys::app::error::NOT_FOUND)),
            RuniqueError::Validation(msg) => {
                info!("{}", tf(runique_i18n::keys::app::error::VALIDATION, &[msg]))
            }
            RuniqueError::Database(msg) => {
                error!("{}", tf(runique_i18n::keys::app::error::DATABASE, &[msg]))
            }
            RuniqueError::Io(msg) => error!("{}", tf(runique_i18n::keys::app::error::IO, &[msg])),
            RuniqueError::Template { message, .. } => {
                error!(
                    "{}",
                    tf(runique_i18n::keys::app::error::TEMPLATE, &[message])
                );
            }
            RuniqueError::Custom { message, source } => {
                error!("{}", tf(runique_i18n::keys::app::error::CUSTOM, &[message]));
                if let Some(source) = source.as_ref() {
                    error!(
                        "{}",
                        tf(
                            runique_i18n::keys::app::error::SOURCE,
                            &[&source.to_string()]
                        )
                    );
                }
            }
        }
    }

    /// Converts the error to `ErrorContext` for rich rendering.
    pub fn to_error_context(&self) -> ErrorContext {
        let (status, error_type, title) = match self {
            RuniqueError::NotFound => (
                StatusCode::NOT_FOUND,
                ErrorType::NotFound,
                ("{}", t(runique_i18n::keys::app::error::NOT_FOUND)),
            ),
            RuniqueError::Forbidden => (
                StatusCode::FORBIDDEN,
                ErrorType::Internal,
                ("{}", t(runique_i18n::keys::app::error::FORBIDDEN)),
            ),
            RuniqueError::Validation(_) => (
                StatusCode::BAD_REQUEST,
                ErrorType::Validation,
                ("{}", t(runique_i18n::keys::app::error::VALIDATION)),
            ),
            RuniqueError::Database(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                ErrorType::Database,
                ("{}", t(runique_i18n::keys::app::error::DATABASE)),
            ),
            RuniqueError::Template { .. } => (
                StatusCode::INTERNAL_SERVER_ERROR,
                ErrorType::Template,
                ("{}", t(runique_i18n::keys::app::error::TEMPLATE)),
            ),
            _ => (
                StatusCode::INTERNAL_SERVER_ERROR,
                ErrorType::Internal,
                ("{}", t(runique_i18n::keys::app::error::INTERNAL)),
            ),
        };

        let mut ctx = ErrorContext::new(error_type, status, &title.1, &self.to_string());
        ctx.build_stack_trace(self);
        ctx
    }
}

impl IntoResponse for RuniqueError {
    fn into_response(self) -> Response {
        self.log();

        // Create a rich ErrorContext instead of a simple message
        let error_context = self.to_error_context();
        let status = StatusCode::from_u16(error_context.status_code)
            .unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);

        // Attach the ErrorContext to the response so the middleware can retrieve it
        let mut response = status.into_response();
        response.extensions_mut().insert(Arc::new(self));
        response
    }
}
