//! Centralized framework errors: `RuniqueError`, `ErrorContext`, and HTML/JSON error rendering.
use crate::middleware::RequestInfoHelper;
use crate::utils::aliases::StrMap;
use crate::utils::env::is_debug;
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Serialize;
use std::sync::Arc;
use std::sync::OnceLock;
use thiserror::Error;
use tracing::{error, info};

use crate::utils::constante::{ADMIN_TEMPLATES, ERROR_CORPS, FIELD_TEMPLATES, SIMPLE_TEMPLATES};
use crate::utils::trad::{t, tf};
// ═══════════════════════════════════════════════════════════════
// BUILD ERRORS (app builder redesign)
// ═══════════════════════════════════════════════════════════════
use crate::app::error_build::BuildError;

static INTERNAL_TEMPLATES: OnceLock<Vec<&'static str>> = OnceLock::new();
fn get_internal_templates() -> &'static [&'static str] {
    INTERNAL_TEMPLATES
        .get_or_init(|| {
            SIMPLE_TEMPLATES
                .iter()
                .chain(ERROR_CORPS.iter())
                .chain(FIELD_TEMPLATES.iter())
                .chain(ADMIN_TEMPLATES.iter())
                .map(|(name, _)| *name)
                .collect::<Vec<&'static str>>()
        })
        .as_slice()
}

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
    #[error("Template error: {0}")]
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
            RuniqueError::Build(e) => RuniqueError::Build(e.clone()),
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

impl From<BuildError> for RuniqueError {
    fn from(err: BuildError) -> Self {
        RuniqueError::Build(err)
    }
}
impl RuniqueError {
    /// Logs the error with the appropriate tracing level (error/info).
    pub fn log(&self) {
        match self {
            RuniqueError::Build(e) => error!("{}", tf("error.build", &[&e.to_string()])),
            RuniqueError::Internal => error!("{}", t("error.internal")),
            RuniqueError::Forbidden => info!("{}", t("error.forbidden")),
            RuniqueError::NotFound => info!("{}", t("error.not_found")),
            RuniqueError::Validation(msg) => info!("{}", tf("error.validation", &[msg])),
            RuniqueError::Database(msg) => error!("{}", tf("error.database", &[msg])),
            RuniqueError::Io(msg) => error!("{}", tf("error.io", &[msg])),
            RuniqueError::Template(msg) => error!("{}", tf("error.template", &[msg])),
            RuniqueError::Custom { message, source } => {
                error!("{}", tf("error.custom", &[message]));
                if let Some(source) = source.as_ref() {
                    error!("{}", tf("error.source", &[&source.to_string()]));
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
                ("{}", t("error.not_found")),
            ),
            RuniqueError::Forbidden => (
                StatusCode::FORBIDDEN,
                ErrorType::Internal,
                ("{}", t("error.forbidden")),
            ),
            RuniqueError::Validation(_) => (
                StatusCode::BAD_REQUEST,
                ErrorType::Validation,
                ("{}", t("error.validation")),
            ),
            RuniqueError::Database(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                ErrorType::Database,
                ("{}", t("error.database")),
            ),
            RuniqueError::Template(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                ErrorType::Template,
                ("{}", t("error.template")),
            ),
            _ => (
                StatusCode::INTERNAL_SERVER_ERROR,
                ErrorType::Internal,
                ("{}", t("error.internal")),
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

// ----------- ERROR CONTEXT (merged from context/error.rs) -----------

/// Rich context of an HTTP error: status, type, debug info, template, request.
/// Attached to response extensions to be rendered by the error middleware.
#[derive(Debug, Serialize, Clone)]
pub struct ErrorContext {
    pub status_code: u16,
    pub error_type: ErrorType,
    pub timestamp: String,
    pub title: String,
    pub message: String,
    /// `{:?}` representation of the root error (full debug format)
    pub debug_repr: Option<String>,
    pub details: Option<String>,
    pub template_info: Option<TemplateInfo>,
    pub request_info: Option<RequestInfo>,
    pub stack_trace: Vec<StackFrame>,
    pub environment: EnvironmentInfo,
}

/// Error category for rendering and logging.
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
    /// `{:?}` representation of this error in the chain
    pub debug_repr: Option<String>,
    pub location: Option<String>,
}

#[derive(Debug, Serialize, Clone)]
pub struct EnvironmentInfo {
    pub debug_mode: bool,
    pub rust_version: String,
    pub app_version: String,
}

impl ErrorContext {
    /// Creates an error context with basic information.
    pub fn new(error_type: ErrorType, status_code: StatusCode, title: &str, message: &str) -> Self {
        Self {
            status_code: status_code.as_u16(),
            error_type,
            timestamp: chrono::Utc::now().to_rfc3339(),
            title: title.to_string(),
            message: message.to_string(),
            debug_repr: None,
            details: None,
            template_info: None,
            request_info: None,
            stack_trace: Vec::new(),
            environment: EnvironmentInfo {
                debug_mode: is_debug(),
                rust_version: rust_version(),
                app_version: env!("CARGO_PKG_VERSION").to_string(),
            },
        }
    }
    #[must_use]
    pub fn with_request_helper(mut self, helper: &RequestInfoHelper) -> Self {
        self.request_info = Some(RequestInfo {
            method: helper.method.clone(),
            path: helper.path.clone(),
            query: helper.query.clone(),
            headers: helper.headers.clone(),
        });
        self
    }
    #[must_use]
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
            &tf("title.template", &[template_name]),
            &error.to_string(),
        );
        ctx.template_info = Some(TemplateInfo {
            name: template_name.to_string(),
            source: read_template_source(template_name),
            line_number: Self::extract_tera_line(error),
            available_templates: tera
                .get_template_names()
                .filter(|name| !get_internal_templates().contains(name))
                .map(std::string::ToString::to_string)
                .collect(),
        });
        ctx.build_stack_trace(error);
        ctx
    }
    pub fn database(error: impl std::error::Error) -> Self {
        let mut ctx = Self::new(
            ErrorType::Database,
            StatusCode::INTERNAL_SERVER_ERROR,
            &tf("error.database", &[&error.to_string()]),
            &error.to_string(),
        );
        ctx.build_stack_trace(&error);
        ctx
    }
    pub fn not_found(path: &str) -> Self {
        Self::new(
            ErrorType::NotFound,
            StatusCode::NOT_FOUND,
            &t("error.title.not_found"),
            &tf("error.path_not_found", &[path]),
        )
    }
    pub fn generic(status: StatusCode, message: &str) -> Self {
        Self::new(
            ErrorType::Internal,
            status,
            &t("error.title.internal"),
            message,
        )
    }
    pub fn from_anyhow(error: &anyhow::Error) -> Self {
        let mut ctx = Self::new(
            ErrorType::Internal,
            StatusCode::INTERNAL_SERVER_ERROR,
            &t("error.AppError"),
            &error.to_string(),
        );
        // Capture the full `{:?}` of the anyhow error (includes chain + backtrace)
        ctx.debug_repr = Some(format!("{error:?}"));

        for (i, cause) in error.chain().enumerate() {
            ctx.stack_trace.push(StackFrame {
                level: i,
                message: cause.to_string(),
                debug_repr: Some(format!("{cause:?}")),
                location: None,
            });
        }
        ctx
    }

    pub fn with_request(mut self, request: &axum::extract::Request) -> Self {
        self.request_info = Some(RequestInfo {
            method: request.method().to_string(),
            path: request.uri().path().to_string(),
            query: request.uri().query().map(std::string::ToString::to_string),
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
    #[must_use]
    pub fn with_details(mut self, details: &str) -> Self {
        self.details = Some(details.to_string());
        self
    }

    pub fn build_stack_trace(&mut self, error: &dyn std::error::Error) {
        // Capture the `{:?}` of the root error on the ErrorContext
        self.debug_repr = Some(format!("{error:?}"));

        let mut level = 0;
        let mut current: Option<&dyn std::error::Error> = Some(error);
        while let Some(err) = current {
            self.stack_trace.push(StackFrame {
                level,
                message: err.to_string(),
                debug_repr: Some(format!("{err:?}")),
                location: None,
            });
            current = err.source();
            level = level.saturating_add(1);
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
            RuniqueError::Build(e) => Self::generic(
                StatusCode::INTERNAL_SERVER_ERROR,
                &tf("error.build", &[&e.to_string()]),
            ),
            RuniqueError::Internal => Self::generic(
                StatusCode::INTERNAL_SERVER_ERROR,
                &t("error.internal_occurred"),
            ),
            RuniqueError::Forbidden => Self::generic(StatusCode::FORBIDDEN, &t("error.forbidden")),
            RuniqueError::NotFound => {
                let path = path.unwrap_or("/");
                Self::not_found(path)
            }
            RuniqueError::Validation(msg) => Self::generic(StatusCode::BAD_REQUEST, msg),
            RuniqueError::Database(msg) => Self::database(sea_orm::DbErr::Custom(msg.clone())),
            RuniqueError::Io(msg) => Self::generic(
                StatusCode::INTERNAL_SERVER_ERROR,
                &tf("error.io", &[msg.as_str()]),
            ),
            RuniqueError::Template(msg) => Self::generic(
                StatusCode::INTERNAL_SERVER_ERROR,
                &tf("error.template", &[msg.as_str()]),
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
                available_templates: tera
                    .get_template_names()
                    .map(std::string::ToString::to_string)
                    .collect(),
            });
        }
        ctx
    }
}
pub fn read_template_source(template_name: &str) -> Option<String> {
    let template_path = format!("templates/{template_name}");
    std::fs::read_to_string(&template_path).ok()
}

fn rust_version() -> String {
    use std::process::Command;
    use std::sync::OnceLock;
    static RUST_VERSION: OnceLock<String> = OnceLock::new();
    RUST_VERSION
        .get_or_init(|| {
            if let Ok(output) = Command::new("rustc").arg("--version").output()
                && let Ok(version) = String::from_utf8(output.stdout)
            {
                return version
                    .split('(')
                    .next()
                    .unwrap_or("N/A")
                    .trim()
                    .to_string();
            }
            "N/A".to_string()
        })
        .clone()
}
