// Liste des templates internes chargés par load_internal_templates
const INTERNAL_TEMPLATES: &[&str] = &[
    "base_index",
    "message",
    "404",
    "500",
    "debug",
    "csrf",
    "csp",
    "errors/corps-error/header-error.html",
    "errors/corps-error/message-error.html",
    "errors/corps-error/template-info.html",
    "errors/corps-error/stack-trace-error.html",
    "errors/corps-error/request-info.html",
    "errors/corps-error/environment-info.html",
    "errors/corps-error/status-code-info.html",
    "errors/corps-error/footer-error.html",
    "base_boolean",
    "base_checkbox",
    "base_color",
    "base_datetime",
    "base_file",
    "base_number",
    "base_radio",
    "base_select",
    "base_special",
    "base_string",
];
use crate::middleware::RequestInfoHelper;
use axum::http::StatusCode;
use serde::Serialize;
use std::collections::HashMap;

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

/// Type d'erreur
#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "lowercase")]
pub enum ErrorType {
    Template,
    NotFound,
    Internal,
    Database,
    Validation,
}

/// Informations sur le template
#[derive(Debug, Serialize, Clone)]
pub struct TemplateInfo {
    pub name: String,
    pub source: Option<String>,
    pub line_number: Option<usize>,
    pub available_templates: Vec<String>,
}

/// Informations sur la requête HTTP
#[derive(Debug, Serialize, Clone)]
pub struct RequestInfo {
    pub method: String,
    pub path: String,
    pub query: Option<String>,
    pub headers: HashMap<String, String>,
}

/// Frame de la stack trace
#[derive(Debug, Serialize, Clone)]
pub struct StackFrame {
    pub level: usize,
    pub message: String,
    pub location: Option<String>,
}

/// Informations sur l'environnement
#[derive(Debug, Serialize, Clone)]
pub struct EnvironmentInfo {
    pub debug_mode: bool,
    pub rust_version: String,
    pub app_version: String,
}

impl ErrorContext {
    /// Crée un nouveau contexte d'erreur
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
        // Cherche "line <num>"
        let re = regex::Regex::new(r"line (\d+)").ok()?;
        re.captures(&msg)
            .and_then(|cap| cap.get(1))
            .and_then(|m| m.as_str().parse::<usize>().ok())
    }
    /// Crée un ErrorContext depuis une erreur Tera
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
                .filter(|s| !INTERNAL_TEMPLATES.contains(s))
                .map(|s| s.to_string())
                .collect(),
        });

        ctx.build_stack_trace(error);
        ctx
    }
    /// Crée un ErrorContext pour une erreur de base de données
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

    /// Crée un ErrorContext pour une erreur 404
    pub fn not_found(path: &str) -> Self {
        Self::new(
            ErrorType::NotFound,
            StatusCode::NOT_FOUND,
            "Page Not Found",
            &format!("The requested path '{}' was not found", path),
        )
    }

    /// Crée un ErrorContext générique
    pub fn generic(status: StatusCode, message: &str) -> Self {
        Self::new(
            ErrorType::Internal,
            status,
            "Internal Server Error",
            message,
        )
    }

    /// Crée un ErrorContext depuis une erreur anyhow
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

    /// Ajoute les informations de requête
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

    /// Ajoute des détails supplémentaires
    pub fn with_details(mut self, details: &str) -> Self {
        self.details = Some(details.to_string());
        self
    }

    /// Construit la stack trace depuis une erreur
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
}

/// Lit le source d'un template
fn read_template_source(template_name: &str) -> Option<String> {
    let template_path = format!("templates/{}", template_name);
    std::fs::read_to_string(&template_path).ok()
}

/// Récupère la version de Rust
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
