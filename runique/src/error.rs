//! Error handling and context management
//!
//! This module provides comprehensive error context tracking for debugging
//! and displaying detailed error pages. It supports various error types including
//! template errors, HTTP errors, database errors, and validation errors.
//!
//! # Examples
//!
//! ```rust
//! use runique::error::ErrorContext;
//! use axum::http::StatusCode;
//!
//! // Create a 404 error context
//! let ctx = ErrorContext::not_found("/missing-page");
//!
//! // Create a generic error context
//! let ctx = ErrorContext::generic(
//!     StatusCode::INTERNAL_SERVER_ERROR,
//!     "Something went wrong"
//! );
//! ```

use axum::http::StatusCode;
use serde::Serialize;
use std::collections::HashMap;

/// Complete error context with all debugging information
///
/// Contains comprehensive information about an error including status code,
/// error type, timestamp, stack trace, request details, and environment info.
///
/// # Examples
///
/// ```rust
/// use runique::error::ErrorContext;
///
/// let ctx = ErrorContext::not_found("/api/users/999");
/// assert_eq!(ctx.status_code, 404);
/// ```
#[derive(Debug, Serialize, Clone)]
pub struct ErrorContext {
    /// HTTP status code
    pub status_code: u16,
    /// Type of error
    pub error_type: ErrorType,
    /// ISO 8601 timestamp when error occurred
    pub timestamp: String,
    /// Human-readable error title
    pub title: String,
    /// Detailed error message
    pub message: String,
    /// Additional details (optional)
    pub details: Option<String>,
    /// Template-related information (if applicable)
    pub template_info: Option<TemplateInfo>,
    /// HTTP request information (if applicable)
    pub request_info: Option<RequestInfo>,
    /// Stack trace frames
    pub stack_trace: Vec<StackFrame>,
    /// Environment information
    pub environment: EnvironmentInfo,
}

/// Type of error
///
/// Categorizes errors for better handling and display.
///
/// # Examples
///
/// ```rust
/// use runique::error::ErrorType;
///
/// let error_type = ErrorType::NotFound;
/// ```
#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "lowercase")]
pub enum ErrorType {
    /// Template rendering error
    Template,
    /// 404 Not Found error
    NotFound,
    /// Internal server error
    Internal,
    /// Database-related error
    Database,
    /// Validation error
    Validation,
}

/// Template-related information for debugging
///
/// Contains details about the template that failed to render,
/// including its source code and available templates.
#[derive(Debug, Serialize, Clone)]
pub struct TemplateInfo {
    /// Name of the template
    pub name: String,
    /// Template source code (if available)
    pub source: Option<String>,
    /// Line number where error occurred (if known)
    pub line_number: Option<usize>,
    /// List of all available templates
    pub available_templates: Vec<String>,
}

/// HTTP request information
///
/// Contains details about the HTTP request that triggered the error.
///
/// # Security
///
/// Sensitive headers (authorization, cookie, token) are automatically filtered out.
#[derive(Debug, Serialize, Clone)]
pub struct RequestInfo {
    /// HTTP method (GET, POST, etc.)
    pub method: String,
    /// Request path
    pub path: String,
    /// Query string (if present)
    pub query: Option<String>,
    /// HTTP headers (sensitive headers excluded)
    pub headers: HashMap<String, String>,
}

/// Stack trace frame
///
/// Represents a single frame in the error stack trace.
#[derive(Debug, Serialize, Clone)]
pub struct StackFrame {
    /// Depth level in the stack (0 = top)
    pub level: usize,
    /// Error message at this level
    pub message: String,
    /// Source code location (if available)
    pub location: Option<String>,
}

/// Environment information
///
/// Contains runtime environment details useful for debugging.
#[derive(Debug, Serialize, Clone)]
pub struct EnvironmentInfo {
    /// Whether debug mode is enabled
    pub debug_mode: bool,
    /// Rust compiler version
    pub rust_version: String,
    /// Application version
    pub app_version: String,
}

impl ErrorContext {
    /// Creates a new error context
    ///
    /// # Arguments
    ///
    /// * `error_type` - Type of error
    /// * `status_code` - HTTP status code
    /// * `title` - Error title
    /// * `message` - Error message
    fn new(error_type: ErrorType, status_code: StatusCode, title: &str, message: &str) -> Self {
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

    /// Creates an ErrorContext from a Tera template error
    ///
    /// Extracts detailed information about the template rendering failure,
    /// including template source and available templates.
    ///
    /// # Arguments
    ///
    /// * `error` - Tera error
    /// * `template_name` - Name of the template that failed
    /// * `tera` - Tera instance
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use runique::error::ErrorContext;
    /// use tera::Tera;
    ///
    /// let tera = Tera::new("templates/**/*").unwrap();
    /// let error = tera.render("missing.html", &tera::Context::new()).unwrap_err();
    /// let ctx = ErrorContext::from_tera_error(&error, "missing.html", &tera);
    /// ```
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
            line_number: None,
            available_templates: tera.get_template_names().map(|s| s.to_string()).collect(),
        });

        ctx.build_stack_trace(error);
        ctx
    }

    /// Creates an ErrorContext for a 404 Not Found error
    ///
    /// # Arguments
    ///
    /// * `path` - Requested path that was not found
    ///
    /// # Examples
    ///
    /// ```rust
    /// use runique::error::ErrorContext;
    ///
    /// let ctx = ErrorContext::not_found("/api/users/999");
    /// assert_eq!(ctx.status_code, 404);
    /// assert_eq!(ctx.title, "Page Not Found");
    /// ```
    pub fn not_found(path: &str) -> Self {
        Self::new(
            ErrorType::NotFound,
            StatusCode::NOT_FOUND,
            "Page Not Found",
            &format!("The requested path '{}' was not found", path),
        )
    }

    /// Creates a generic ErrorContext
    ///
    /// # Arguments
    ///
    /// * `status` - HTTP status code
    /// * `message` - Error message
    ///
    /// # Examples
    ///
    /// ```rust
    /// use runique::error::ErrorContext;
    /// use axum::http::StatusCode;
    ///
    /// let ctx = ErrorContext::generic(
    ///     StatusCode::BAD_REQUEST,
    ///     "Invalid input data"
    /// );
    /// ```
    pub fn generic(status: StatusCode, message: &str) -> Self {
        Self::new(
            ErrorType::Internal,
            status,
            "Internal Server Error",
            message,
        )
    }

    /// Creates an ErrorContext from an anyhow error
    ///
    /// Automatically builds the stack trace from the error chain.
    ///
    /// # Arguments
    ///
    /// * `error` - Anyhow error
    ///
    /// # Examples
    ///
    /// ```rust
    /// use runique::error::ErrorContext;
    /// use anyhow::anyhow;
    ///
    /// let error = anyhow!("Database connection failed");
    /// let ctx = ErrorContext::from_anyhow(&error);
    /// ```
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

    /// Adds HTTP request information to the context
    ///
    /// Sensitive headers (authorization, cookie, token) are automatically filtered out.
    ///
    /// # Arguments
    ///
    /// * `request` - HTTP request
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use runique::error::ErrorContext;
    /// use axum::extract::Request;
    ///
    /// # async fn handler(request: Request) {
    /// let ctx = ErrorContext::not_found("/missing")
    ///     .with_request(&request);
    /// # }
    /// ```
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

    /// Adds additional details to the error context
    ///
    /// # Arguments
    ///
    /// * `details` - Additional details
    ///
    /// # Examples
    ///
    /// ```rust
    /// use runique::error::ErrorContext;
    /// use axum::http::StatusCode;
    ///
    /// let ctx = ErrorContext::generic(StatusCode::BAD_REQUEST, "Invalid data")
    ///     .with_details("Expected JSON, got XML");
    /// ```
    pub fn with_details(mut self, details: &str) -> Self {
        self.details = Some(details.to_string());
        self
    }

    /// Builds the stack trace from an error
    ///
    /// Walks through the error chain and creates stack frames.
    fn build_stack_trace(&mut self, error: &dyn std::error::Error) {
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
}

/// Reads template source code from disk
///
/// # Arguments
///
/// * `template_name` - Name of the template file
///
/// # Returns
///
/// Template source code if file exists and is readable
fn read_template_source(template_name: &str) -> Option<String> {
    let template_path = format!("templates/{}", template_name);
    std::fs::read_to_string(&template_path).ok()
}

/// Gets the Rust compiler version
///
/// Caches the result after first call for performance.
///
/// # Returns
///
/// Rust version string (e.g., "rustc 1.75.0") or "N/A" if unavailable
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
