//! `ErrorContext`: what an error page is rendered from — status, type, message,
//! and in debug the template, the request and the chain of causes.
use super::debug_page::read_template_source;
use super::error::{RuniqueError, tera_error_line};
use super::request_snapshot::RequestSnapshot;
use crate::config::env::is_debug;
use crate::templates::constants::{
    ADMIN_TEMPLATES, ERROR_CORPS, FIELD_TEMPLATES, SIMPLE_TEMPLATES,
};
use crate::translation::{t, tf};
use crate::utils::aliases::StrMap;
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Serialize;
use std::sync::{Arc, OnceLock};

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
    /// A request the client got wrong (e.g. a query string that doesn't
    /// deserialize): rendered as `400.html`.
    BadRequest,
}

/// Diagnostic details about a failed template render, used to build the debug error page.
#[derive(Debug, Serialize, Clone)]
pub struct TemplateInfo {
    /// Name of the template that failed to render.
    pub name: String,
    /// Raw source of the template, read from disk when available.
    pub source: Option<String>,
    /// Line number where rendering failed, parsed from the Tera error message.
    pub line_number: Option<usize>,
    /// Names of templates known to Tera, excluding Runique's internal/builtin templates.
    pub available_templates: Vec<String>,
}

/// HTTP request metadata captured for the debug error page.
#[derive(Debug, Serialize, Clone)]
pub struct RequestInfo {
    pub method: String,
    pub path: String,
    pub query: Option<String>,
    /// Request headers with `authorization`, `cookie`, and `token` entries stripped.
    pub headers: StrMap,
}

/// One level of the underlying error chain, captured via `Error::source()`.
#[derive(Debug, Serialize, Clone)]
pub struct StackFrame {
    /// Depth in the error chain, starting at 0 for the top-level error.
    pub level: usize,
    pub message: String,
    /// `{:?}` representation of this error in the chain
    pub debug_repr: Option<String>,
    /// Source location of this error, when available (currently always `None`).
    pub location: Option<String>,
}

/// Runtime environment metadata shown on the debug error page.
#[derive(Debug, Serialize, Clone)]
pub struct EnvironmentInfo {
    pub debug_mode: bool,
    /// The compiler that built the binary, captured by `build.rs`.
    pub rust_version: String,
    pub runique_version: String,
}

impl ErrorContext {
    /// A bare response of this context's status, carrying the context: the
    /// error middleware renders the page (`{code}.html` in production, the
    /// debug page with the reason in debug).
    pub(crate) fn into_error_response(self) -> Response {
        let status =
            StatusCode::from_u16(self.status_code).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
        let mut response = status.into_response();
        response.extensions_mut().insert(Arc::new(self));
        response
    }

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
                rust_version: env!("RUNIQUE_RUSTC_VERSION").to_string(),
                runique_version: env!("CARGO_PKG_VERSION").to_string(),
            },
        }
    }
    /// Attaches request info from a `RequestSnapshot` snapshot, for use when the
    /// original `Request` is no longer available (e.g. from within middleware).
    #[must_use]
    pub fn with_request_snapshot(mut self, helper: &RequestSnapshot) -> Self {
        self.request_info = Some(RequestInfo {
            method: helper.method.clone(),
            path: helper.path.clone(),
            query: helper.query.clone(),
            headers: helper.headers.clone(),
        });
        self
    }
    /// Builds an `ErrorContext` from a Tera rendering error, resolving the failing
    /// template's source and the line number read from the error's span.
    pub fn from_tera_error(error: &tera::Error, template_name: &str, tera: &tera::Tera) -> Self {
        let mut ctx = Self::new(
            ErrorType::Template,
            StatusCode::INTERNAL_SERVER_ERROR,
            &tf(
                runique_i18n::keys::app::label::TEMPLATE_IN,
                &[template_name],
            ),
            &error.to_string(),
        );
        ctx.template_info = Some(TemplateInfo {
            name: template_name.to_string(),
            source: read_template_source(template_name),
            line_number: tera_error_line(error),
            available_templates: tera
                .get_template_names()
                .filter(|name| !get_internal_templates().contains(name))
                .map(std::string::ToString::to_string)
                .collect(),
        });
        ctx.build_stack_trace(error);
        ctx
    }
    /// Builds an `ErrorContext` for a database error, capturing its full source chain.
    pub fn database(error: impl std::error::Error) -> Self {
        let mut ctx = Self::new(
            ErrorType::Database,
            StatusCode::INTERNAL_SERVER_ERROR,
            &tf(
                runique_i18n::keys::app::error::DATABASE,
                &[&error.to_string()],
            ),
            &error.to_string(),
        );
        ctx.build_stack_trace(&error);
        ctx
    }
    /// Builds a 404 `ErrorContext` for the given request path.
    pub fn not_found(path: &str) -> Self {
        Self::new(
            ErrorType::NotFound,
            StatusCode::NOT_FOUND,
            &t(runique_i18n::keys::app::label::NOT_FOUND),
            &tf(runique_i18n::keys::app::error::PATH_NOT_FOUND, &[path]),
        )
    }
    /// Builds a 400 `ErrorContext`: the request itself is malformed. `message`
    /// is shown on the debug page only, never in production.
    pub fn bad_request(message: &str) -> Self {
        let mut ctx = Self::new(
            ErrorType::BadRequest,
            StatusCode::BAD_REQUEST,
            &t(runique_i18n::keys::app::label::BAD_REQUEST),
            message,
        );
        // The debug page shows the first frame as the reason.
        ctx.stack_trace.push(StackFrame {
            level: 0,
            message: message.to_string(),
            debug_repr: None,
            location: None,
        });
        ctx
    }
    /// Builds an `ErrorContext` with an arbitrary status code and the generic `Internal` type.
    pub fn generic(status: StatusCode, message: &str) -> Self {
        Self::new(
            ErrorType::Internal,
            status,
            &t(runique_i18n::keys::app::label::INTERNAL),
            message,
        )
    }
    /// Builds an `ErrorContext` from an `anyhow::Error`, capturing its full debug
    /// representation and recording each cause in its chain as a stack frame.
    pub fn from_anyhow(error: &anyhow::Error) -> Self {
        let mut ctx = Self::new(
            ErrorType::Internal,
            StatusCode::INTERNAL_SERVER_ERROR,
            &t(runique_i18n::keys::app::error::APP_ERROR),
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

    /// Walks the error's `source()` chain, recording one `StackFrame` per level and
    /// the root error's `{:?}` debug representation.
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
    /// Builds an `ErrorContext` from a `RuniqueError`, mapping each variant to a
    /// status code and title, then enriching it with request and template info when available.
    pub fn from_runique_error(
        err: &RuniqueError,
        path: Option<&str>,
        request_snapshot: Option<&RequestSnapshot>,
        template_name: Option<&str>,
        tera: Option<&tera::Tera>,
    ) -> Self {
        let mut ctx = match err {
            RuniqueError::Build(e) => Self::generic(
                StatusCode::INTERNAL_SERVER_ERROR,
                &tf(runique_i18n::keys::app::error::BUILD, &[&e.to_string()]),
            ),
            RuniqueError::Internal => Self::generic(
                StatusCode::INTERNAL_SERVER_ERROR,
                &t(runique_i18n::keys::app::error::INTERNAL_OCCURRED),
            ),
            RuniqueError::Forbidden => Self::generic(
                StatusCode::FORBIDDEN,
                &t(runique_i18n::keys::app::error::FORBIDDEN),
            ),
            RuniqueError::NotFound => {
                let path = path.unwrap_or("/");
                Self::not_found(path)
            }
            RuniqueError::Validation(msg) => Self::generic(StatusCode::BAD_REQUEST, msg),
            RuniqueError::Database(msg) => Self::database(sea_orm::DbErr::Custom(msg.clone())),
            RuniqueError::Io(msg) => Self::generic(
                StatusCode::INTERNAL_SERVER_ERROR,
                &tf(runique_i18n::keys::app::error::IO, &[msg.as_str()]),
            ),
            RuniqueError::Template { message, .. } => Self::generic(
                StatusCode::INTERNAL_SERVER_ERROR,
                &tf(
                    runique_i18n::keys::app::error::TEMPLATE,
                    &[message.as_str()],
                ),
            ),
            RuniqueError::Custom { message, source: _ } => {
                Self::generic(StatusCode::INTERNAL_SERVER_ERROR, message)
            }
        };

        if let Some(helper) = request_snapshot {
            ctx = ctx.with_request_snapshot(helper);
        }

        ctx.build_stack_trace(err);

        if let (RuniqueError::Template { line, .. }, Some(tera), Some(name)) =
            (err, tera, template_name)
        {
            ctx.template_info = Some(TemplateInfo {
                name: name.to_string(),
                source: read_template_source(name),
                // The line is carried on the variant, filled by
                // `RuniqueError::template` from the Tera error's span.
                line_number: *line,
                available_templates: tera
                    .get_template_names()
                    .map(std::string::ToString::to_string)
                    .collect(),
            });
        }
        ctx
    }
}
