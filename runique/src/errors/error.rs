use crate::middleware::RequestInfoHelper;
use crate::utils::aliases::StrMap;
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Serialize;
use std::sync::Arc;
use std::sync::OnceLock;
use thiserror::Error;
use tracing::{error, info};

use crate::utils::constante::{ERROR_CORPS, FIELD_TEMPLATES, SIMPLE_TEMPLATES};

static INTERNAL_TEMPLATES: OnceLock<Vec<&'static str>> = OnceLock::new();
fn get_internal_templates() -> &'static [&'static str] {
    INTERNAL_TEMPLATES
        .get_or_init(|| {
            SIMPLE_TEMPLATES
                .iter()
                .chain(ERROR_CORPS.iter())
                .chain(FIELD_TEMPLATES.iter())
                .map(|(name, _)| *name)
                .collect::<Vec<&'static str>>()
        })
        .as_slice()
}

// Type Result global pour Runique
pub type RuniqueResult<T> = Result<T, RuniqueError>;

// Erreurs applicatives centralisées
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
    Io(String),
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

impl RuniqueError {
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

    /// Convertit l'erreur en ErrorContext pour un rendu riche
    pub fn to_error_context(&self) -> ErrorContext {
        let (status, error_type, title) = match self {
            RuniqueError::NotFound => {
                (StatusCode::NOT_FOUND, ErrorType::NotFound, "Page Not Found")
            }
            RuniqueError::Forbidden => (
                StatusCode::FORBIDDEN,
                ErrorType::Internal,
                "Access Forbidden",
            ),
            RuniqueError::Validation(_) => (
                StatusCode::BAD_REQUEST,
                ErrorType::Validation,
                "Validation Error",
            ),
            RuniqueError::Database(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                ErrorType::Database,
                "Database Error",
            ),
            RuniqueError::Template(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                ErrorType::Template,
                "Template Error",
            ),
            _ => (
                StatusCode::INTERNAL_SERVER_ERROR,
                ErrorType::Internal,
                "Internal Server Error",
            ),
        };

        let mut ctx = ErrorContext::new(error_type, status, title, &self.to_string());
        ctx.build_stack_trace(self);
        ctx
    }
}

impl IntoResponse for RuniqueError {
    fn into_response(self) -> Response {
        self.log();

        // Créer un ErrorContext riche au lieu d'un simple message
        let error_context = self.to_error_context();
        let status = StatusCode::from_u16(error_context.status_code)
            .unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);

        // Attacher l'ErrorContext à la réponse pour que le middleware puisse le récupérer
        let mut response = status.into_response();
        response.extensions_mut().insert(Arc::new(self));
        response
    }
}

// ----------- CONTEXTE ERREUR (fusionné depuis context/error.rs) -----------

/// Contexte complet pour les erreurs avec toutes les informations de débogage
#[derive(Debug, Serialize, Clone)]
pub struct ErrorContext {
    pub status_code: u16,
    pub error_type: ErrorType,
    pub timestamp: String,
    pub title: String,
    pub message: String,
    pub details: Option<String>,
    pub template_info: Option<TemplateInfo>,
    pub request_info: Option<RequestInfo>,
    pub stack_trace: Vec<StackFrame>,
    pub environment: EnvironmentInfo,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "lowercase")]
pub enum ErrorType {
    Template,
    NotFound,
    Internal,
    Database,
    Validation,
}

#[derive(Debug, Serialize, Clone)]
pub struct TemplateInfo {
    pub name: String,
    pub source: Option<String>,
    pub line_number: Option<usize>,
    pub available_templates: Vec<String>,
}

#[derive(Debug, Serialize, Clone)]
pub struct RequestInfo {
    pub method: String,
    pub path: String,
    pub query: Option<String>,
    pub headers: StrMap,
}

#[derive(Debug, Serialize, Clone)]
pub struct StackFrame {
    pub level: usize,
    pub message: String,
    pub location: Option<String>,
}

#[derive(Debug, Serialize, Clone)]
pub struct EnvironmentInfo {
    pub debug_mode: bool,
    pub rust_version: String,
    pub app_version: String,
}

impl ErrorContext {
    pub fn new(error_type: ErrorType, status_code: StatusCode, title: &str, message: &str) -> Self {
        Self {
            status_code: status_code.as_u16(),
            error_type,
            timestamp: chrono::Utc::now().to_rfc3339(),
            title: title.to_string(),
            message: message.to_string(),
            details: None,
            template_info: None,
            request_info: None,
            stack_trace: Vec::new(),
            environment: EnvironmentInfo {
                debug_mode: cfg!(debug_assertions),
                rust_version: rust_version(),
                app_version: env!("CARGO_PKG_VERSION").to_string(),
            },
        }
    }

    pub fn with_request_helper(mut self, helper: &RequestInfoHelper) -> Self {
        self.request_info = Some(RequestInfo {
            method: helper.method.clone(),
            path: helper.path.clone(),
            query: helper.query.clone(),
            headers: helper.headers.clone(),
        });
        self
    }

    fn extract_tera_line(error: &tera::Error) -> Option<usize> {
        let msg = error.to_string();
        let re = regex::Regex::new(r"line (\d+)").ok()?;
        re.captures(&msg)
            .and_then(|cap| cap.get(1))
            .and_then(|m| m.as_str().parse::<usize>().ok())
    }

    pub fn from_tera_error(error: &tera::Error, template_name: &str, tera: &tera::Tera) -> Self {
        let mut ctx = Self::new(
            ErrorType::Template,
            StatusCode::INTERNAL_SERVER_ERROR,
            "Template Rendering Error",
            &error.to_string(),
        );
        ctx.template_info = Some(TemplateInfo {
            name: template_name.to_string(),
            source: read_template_source(template_name),
            line_number: Self::extract_tera_line(error),
            available_templates: tera
                .get_template_names()
                .filter(|name| !get_internal_templates().contains(name))
                .map(|s| s.to_string())
                .collect(),
        });
        ctx.build_stack_trace(error);
        ctx
    }

    pub fn database(error: impl std::error::Error) -> Self {
        let mut ctx = Self::new(
            ErrorType::Database,
            StatusCode::INTERNAL_SERVER_ERROR,
            "Database Error",
            &error.to_string(),
        );
        ctx.build_stack_trace(&error);
        ctx
    }

    pub fn not_found(path: &str) -> Self {
        Self::new(
            ErrorType::NotFound,
            StatusCode::NOT_FOUND,
            "Page Not Found",
            &format!("The requested path '{}' was not found", path),
        )
    }

    pub fn generic(status: StatusCode, message: &str) -> Self {
        Self::new(
            ErrorType::Internal,
            status,
            "Internal Server Error",
            message,
        )
    }

    pub fn from_anyhow(error: &anyhow::Error) -> Self {
        let mut ctx = Self::new(
            ErrorType::Internal,
            StatusCode::INTERNAL_SERVER_ERROR,
            "Application Error",
            &error.to_string(),
        );
        for (i, cause) in error.chain().enumerate() {
            ctx.stack_trace.push(StackFrame {
                level: i,
                message: cause.to_string(),
                location: None,
            });
        }
        ctx
    }

    pub fn with_request(mut self, request: &axum::extract::Request) -> Self {
        self.request_info = Some(RequestInfo {
            method: request.method().to_string(),
            path: request.uri().path().to_string(),
            query: request.uri().query().map(|q| q.to_string()),
            headers: request
                .headers()
                .iter()
                .filter(|(k, _)| {
                    let key = k.as_str().to_lowercase();
                    !key.contains("authorization")
                        && !key.contains("cookie")
                        && !key.contains("token")
                })
                .map(|(k, v)| (k.to_string(), v.to_str().unwrap_or("").to_string()))
                .collect(),
        });
        self
    }

    pub fn with_details(mut self, details: &str) -> Self {
        self.details = Some(details.to_string());
        self
    }

    pub fn build_stack_trace(&mut self, error: &dyn std::error::Error) {
        let mut level = 0;
        let mut current: Option<&dyn std::error::Error> = Some(error);
        while let Some(err) = current {
            self.stack_trace.push(StackFrame {
                level,
                message: err.to_string(),
                location: None,
            });
            current = err.source();
            level += 1;
        }
    }

    pub fn from_runique_error(
        err: &RuniqueError,
        path: Option<&str>,
        request_helper: Option<&RequestInfoHelper>,
        template_name: Option<&str>,
        tera: Option<&tera::Tera>,
    ) -> Self {
        let mut ctx = match err {
            RuniqueError::Internal => Self::generic(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Une erreur interne est survenue",
            ),
            RuniqueError::Forbidden => Self::generic(StatusCode::FORBIDDEN, "Accès interdit"),
            RuniqueError::NotFound => {
                let path = path.unwrap_or("/");
                Self::not_found(path)
            }
            RuniqueError::Validation(msg) => Self::generic(StatusCode::BAD_REQUEST, msg),
            RuniqueError::Database(msg) => Self::database(sea_orm::DbErr::Custom(msg.clone())),
            RuniqueError::Io(msg) => Self::generic(
                StatusCode::INTERNAL_SERVER_ERROR,
                &format!("IO Error: {}", msg),
            ),
            RuniqueError::Template(msg) => Self::generic(
                StatusCode::INTERNAL_SERVER_ERROR,
                &format!("Template Error: {}", msg),
            ),
            RuniqueError::Custom { message, source: _ } => {
                Self::generic(StatusCode::INTERNAL_SERVER_ERROR, message)
            }
        };

        if let Some(helper) = request_helper {
            ctx = ctx.with_request_helper(helper);
        }

        ctx.build_stack_trace(err);

        if let (RuniqueError::Template(_), Some(tera), Some(name)) = (err, tera, template_name) {
            ctx.template_info = Some(TemplateInfo {
                name: name.to_string(),
                source: read_template_source(name),
                line_number: ErrorContext::extract_tera_line(&tera.get_template(name).unwrap_err()),
                available_templates: tera.get_template_names().map(|s| s.to_string()).collect(),
            });
        }
        ctx
    }
}

pub fn read_template_source(template_name: &str) -> Option<String> {
    let template_path = format!("templates/{}", template_name);
    std::fs::read_to_string(&template_path).ok()
}

fn rust_version() -> String {
    use std::process::Command;
    use std::sync::OnceLock;
    static RUST_VERSION: OnceLock<String> = OnceLock::new();
    RUST_VERSION
        .get_or_init(|| {
            if let Ok(output) = Command::new("rustc").arg("--version").output() {
                if let Ok(version) = String::from_utf8(output.stdout) {
                    return version
                        .split('(')
                        .next()
                        .unwrap_or("N/A")
                        .trim()
                        .to_string();
                }
            }
            "N/A".to_string()
        })
        .clone()
}
