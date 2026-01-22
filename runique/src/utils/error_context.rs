use axum::http::StatusCode;
use serde::Serialize;
use std::collections::HashMap;

/// Complete error context with all debugging information
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

/// Type of error
#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "lowercase")]
pub enum ErrorType {
    Template,
    NotFound,
    Internal,
    Database,
    Validation,
}

/// Template-related information for debugging
#[derive(Debug, Serialize, Clone)]
pub struct TemplateInfo {
    pub name: String,
    pub source: Option<String>,
    pub line_number: Option<usize>,
    pub available_templates: Vec<String>,
}

/// HTTP request information
#[derive(Debug, Serialize, Clone)]
pub struct RequestInfo {
    pub method: String,
    pub path: String,
    pub query: Option<String>,
    pub headers: HashMap<String, String>,
}

/// Stack trace frame
#[derive(Debug, Serialize, Clone)]
pub struct StackFrame {
    pub level: usize,
    pub message: String,
    pub location: Option<String>,
}

/// Environment information
#[derive(Debug, Serialize, Clone)]
pub struct EnvironmentInfo {
    pub debug_mode: bool,
    pub rust_version: String,
    pub app_version: String,
}

impl ErrorContext {
    /// Creates a new error context
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
    fn extract_tera_line(error: &tera::Error) -> Option<usize> {
        let msg = error.to_string();
        // Cherche "line <num>"
        let re = regex::Regex::new(r"line (\d+)").ok()?;
        re.captures(&msg)
            .and_then(|cap| cap.get(1))
            .and_then(|m| m.as_str().parse::<usize>().ok())
    }
    /// ErrorContext from a Tera template error
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
            line_number: Self::extract_tera_line(error), // récupère ligne si disponible
            available_templates: tera.get_template_names().map(|s| s.to_string()).collect(),
        });

        ctx.build_stack_trace(error);
        ctx
    }

    /// 404 Not Found
    pub fn not_found(path: &str) -> Self {
        Self::new(
            ErrorType::NotFound,
            StatusCode::NOT_FOUND,
            "Page Not Found",
            &format!("The requested path '{}' was not found", path),
        )
    }

    /// Generic error
    pub fn generic(status: StatusCode, message: &str) -> Self {
        Self::new(ErrorType::Internal, status, "Internal Server Error", message)
    }

    /// From anyhow error
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

    /// Adds request information
    pub fn with_request(mut self, request: &axum::http::Request<axum::body::Body>) -> Self {
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

    /// Manual request info
    pub fn with_request_info(
        mut self,
        method: String,
        path: String,
        query: Option<String>,
    ) -> Self {
        self.request_info = Some(RequestInfo {
            method,
            path,
            query,
            headers: HashMap::new(),
        });
        self
    }

    /// Add details
    pub fn with_details(mut self, details: &str) -> Self {
        self.details = Some(details.to_string());
        self
    }

    /// Build stack trace
    fn build_stack_trace(&mut self, error: &dyn std::error::Error) {
        let mut level = 0;
        let mut current: Option<&dyn std::error::Error> = Some(error);

        while let Some(err) = current {
            self.stack_trace.push(StackFrame {
                level,
                message: err.to_string(),
                location: None, // si possible : fichier + ligne
            });
            current = err.source();
            level += 1;
        }
    }
}

/// Read template source from disk
fn read_template_source(template_name: &str) -> Option<String> {
    let possible_paths = vec![
        format!("templates/{}", template_name),
        format!("src/templates/{}", template_name),
        format!("./templates/{}", template_name),
    ];

    for path in possible_paths {
        if let Ok(content) = std::fs::read_to_string(&path) {
            return Some(content);
        }
    }

    None
}

/// Get Rust compiler version
fn rust_version() -> String {
    use std::process::Command;
    use std::sync::OnceLock;

    static RUST_VERSION: OnceLock<String> = OnceLock::new();

    RUST_VERSION
        .get_or_init(|| {
            if let Ok(output) = Command::new("rustc").arg("--version").output() {
                if let Ok(version) = String::from_utf8(output.stdout) {
                    return version.split('(').next().unwrap_or("N/A").trim().to_string();
                }
            }
            "N/A".to_string()
        })
        .clone()
}
