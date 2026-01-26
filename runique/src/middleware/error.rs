use axum::{
    extract::Extension,
    http::{Request, StatusCode},
    middleware::Next,
    response::{Html, IntoResponse, Response},
};
use chrono::Utc;
use std::collections::HashMap;
use std::sync::Arc;
use tera::{Context, Tera};

use crate::{config::RuniqueConfig, context::error::ErrorContext, utils::csrf::CsrfToken};

/// Middleware d’erreur centralisé avec Stack Trace et Debug contextuel
pub async fn error_handler_middleware(
    Extension(tera): Extension<Arc<Tera>>,
    Extension(config): Extension<Arc<RuniqueConfig>>,
    request: Request<axum::body::Body>,
    next: Next,
) -> Response {
    // 1. Capture des informations de la requête avant exécution
    let csrf_token: Option<String> = request.extensions().get::<CsrfToken>().map(|t| t.0.clone());

    let method = request.method().to_string();
    let path = request.uri().path().to_string();
    let query = request.uri().query().map(|q| q.to_string());

    // Capture sécurisée des headers (on exclut les données sensibles)
    let mut headers = HashMap::new();
    for (name, value) in request.headers() {
        let name_str = name.as_str().to_lowercase();
        if !name_str.contains("cookie")
            && !name_str.contains("authorization")
            && !name_str.contains("token")
        {
            headers.insert(
                name.to_string(),
                value.to_str().unwrap_or("[Non-ASCII Header]").to_string(),
            );
        }
    }

    let request_helper = RequestInfoHelper {
        method,
        path: path.clone(),
        query,
        headers,
    };
    let response = next.run(request).await;
    let status = response.status();
    // 2. Exécution du cycle de vie de la requête
    if status.is_server_error() || status == StatusCode::NOT_FOUND {
        if config.debug {
            // Essaie de récupérer le ErrorContext depuis les extensions
            let error_ctx = response
                .extensions()
                .get::<Arc<ErrorContext>>() // ← Change ici
                .map(|ctx| (**ctx).clone())
                .unwrap_or_else(|| {
                    if status == StatusCode::NOT_FOUND {
                        ErrorContext::not_found(&path)
                    } else {
                        ErrorContext::generic(status, "Une erreur interne est survenue")
                    }
                })
                .with_request_helper(&request_helper);

            return render_debug_error_from_context(&tera, &config, error_ctx, csrf_token);
        } else {
            return match status {
                StatusCode::NOT_FOUND => render_404(&tera, &config, csrf_token),
                _ => render_500(&tera, &config, csrf_token),
            };
        }
    }

    response
}

/// Helper pour transporter les infos de requête vers ErrorContext
pub struct RequestInfoHelper {
    pub method: String,
    pub path: String,
    pub query: Option<String>,
    pub headers: HashMap<String, String>,
}

/// Rend la page 404 (Production)
pub fn render_404(tera: &Tera, config: &RuniqueConfig, csrf_token: Option<String>) -> Response {
    let mut context = Context::new();
    inject_global_vars(&mut context, config, csrf_token);

    match tera.render("404", &context) {
        Ok(html) => (StatusCode::NOT_FOUND, Html(html)).into_response(),
        Err(_) => fallback_404_html(),
    }
}

/// Rend la page 500 (Production)
pub fn render_500(tera: &Tera, config: &RuniqueConfig, csrf_token: Option<String>) -> Response {
    let mut context = Context::new();
    inject_global_vars(&mut context, config, csrf_token);

    match tera.render("500", &context) {
        Ok(html) => (StatusCode::INTERNAL_SERVER_ERROR, Html(html)).into_response(),
        Err(_) => fallback_500_html(),
    }
}

/// Rend la page de debug riche (Développement)
fn render_debug_error_from_context(
    tera: &Tera,
    config: &RuniqueConfig,
    error_ctx: ErrorContext,
    csrf_token: Option<String>,
) -> Response {
    let mut context = match Context::from_serialize(&error_ctx) {
        Ok(ctx) => ctx,
        Err(e) => return critical_error_html(&format!("Serialization Error: {}", e)),
    };

    inject_global_vars(&mut context, config, csrf_token);

    match tera.render("debug", &context) {
        Ok(html) => (
            StatusCode::from_u16(error_ctx.status_code)
                .unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Html(html),
        )
            .into_response(),
        Err(e) => critical_error_html(&format!("Tera Rendering Error: {}", e)),
    }
}

/// Injecte les variables communes à tous les templates d'erreur
fn inject_global_vars(context: &mut Context, config: &RuniqueConfig, csrf_token: Option<String>) {
    context.insert("static_runique", &config.static_files.static_runique_url);
    context.insert("timestamp", &Utc::now().to_rfc3339());
    if let Some(token) = csrf_token {
        context.insert("csrf_token", &token);
    }
}

// --- FALLBACKS ---

fn fallback_404_html() -> Response {
    let html = r#"<!DOCTYPE html><html><head><title>404</title></head><body><h1>404 - Not Found</h1></body></html>"#;
    (StatusCode::NOT_FOUND, Html(html)).into_response()
}

fn fallback_500_html() -> Response {
    let html = r#"<!DOCTYPE html><html><head><title>500</title></head><body><h1>500 - Server Error</h1></body></html>"#;
    (StatusCode::INTERNAL_SERVER_ERROR, Html(html)).into_response()
}

fn critical_error_html(error: &str) -> Response {
    let html = format!(
        r#"<!DOCTYPE html><html><head><title>Critical Error</title></head>
        <body style="font-family:sans-serif;padding:2rem;background:#fff5f5;">
        <h1 style="color:#c53030;">Critical Error</h1>
        <p>The error reporting system itself failed.</p>
        <pre style="background:#fff;padding:1rem;border:1px solid #feb2b2;">{}</pre>
        </body></html>"#,
        html_escape(error)
    );
    (StatusCode::INTERNAL_SERVER_ERROR, Html(html)).into_response()
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#x27;")
}
