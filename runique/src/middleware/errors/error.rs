use crate::utils::aliases::{ARuniqueConfig, ATera, StrMap};
use axum::{
    extract::Extension,
    http::{Request, StatusCode},
    middleware::Next,
    response::{Html, IntoResponse, Response},
};
use chrono::Utc;
use std::sync::Arc;
use tera::{Context, Tera};
use tracing::{error, info, instrument};
use tracing_futures::Instrument;

use crate::{
    config::RuniqueConfig,
    errors::error::{ErrorContext, RuniqueError},
    utils::csrf::CsrfToken,
};

/// Transport des infos requête pour debug contextuel
pub struct RequestInfoHelper {
    pub method: String,
    pub path: String,
    pub query: Option<String>,
    pub headers: StrMap,
}

/// Middleware principal Runique avec tracing + debug
#[instrument(name = "RuniqueRequest", skip(tera, config, next))]
pub async fn error_handler_middleware(
    Extension(tera): Extension<ATera>,
    Extension(config): Extension<ARuniqueConfig>,
    request: Request<axum::body::Body>,
    next: Next,
) -> Response {
    // ---  Collecte des infos requête ---
    let csrf_token: Option<String> = request.extensions().get::<CsrfToken>().map(|t| t.0.clone());
    let request_helper = RequestInfoHelper {
        method: request.method().to_string(),
        path: request.uri().path().to_string(),
        query: request.uri().query().map(|q| q.to_string()),
        headers: request
            .headers()
            .iter()
            .filter(|(k, _)| {
                let key = k.as_str().to_lowercase();
                !key.contains("authorization") && !key.contains("cookie") && !key.contains("token")
            })
            .map(|(k, v)| (k.to_string(), v.to_str().unwrap_or("").to_string()))
            .collect(),
    };

    // ---  Exécute la requête dans le span tracing ---
    let span = tracing::Span::current();
    let response = next.run(request).instrument(span.clone()).await;

    let status = response.status();

    // ---  Gestion des erreurs ---
    if status.is_server_error() || status == StatusCode::NOT_FOUND {
        // Récupère l'erreur attachée à la réponse si elle existe
        let error_ctx_opt = response
            .extensions()
            .get::<Arc<RuniqueError>>()
            .map(|err| (**err).clone());

        let error_ctx = error_ctx_opt.unwrap_or_else(|| {
            if status == StatusCode::NOT_FOUND {
                RuniqueError::NotFound
            } else {
                RuniqueError::Internal
            }
        });

        // ---  Logging intelligent ---
        match &error_ctx {
            RuniqueError::Internal
            | RuniqueError::Database(_)
            | RuniqueError::Io(_)
            | RuniqueError::Template(_)
            | RuniqueError::Custom { .. } => {
                error!(method = %request_helper.method, path = %request_helper.path, ?error_ctx, "Critical error occurred");
            }
            RuniqueError::Validation(_) | RuniqueError::Forbidden | RuniqueError::NotFound => {
                info!(method = %request_helper.method, path = %request_helper.path, ?error_ctx, "Handled error");
            }
        }

        // ---  Crée un contexte enrichi pour templates ---
        if config.debug {
            let ctx = ErrorContext::from_runique_error(
                &error_ctx,
                Some(&request_helper.path),
                Some(&request_helper),
                None,        // nom du template si applicable
                Some(&tera), // tera pour template_info si template error
            );

            return render_debug_error_from_context(&tera, &config, ctx, csrf_token);
        } else {
            // Production : 404 ou 500 simple
            return match error_ctx {
                RuniqueError::NotFound => render_404(&tera, &config, csrf_token),
                _ => render_500(&tera, &config, csrf_token),
            };
        }
    }

    // ---  Pas d'erreur : retourne la réponse normale ---
    response
}

// --- Render Helpers ---

fn render_404(tera: &Tera, config: &RuniqueConfig, csrf_token: Option<String>) -> Response {
    let mut context = Context::new();
    inject_global_vars(&mut context, config, csrf_token);

    match tera.render("404", &context) {
        Ok(html) => (StatusCode::NOT_FOUND, Html(html)).into_response(),
        Err(_) => fallback_404_html(),
    }
}

fn render_500(tera: &Tera, config: &RuniqueConfig, csrf_token: Option<String>) -> Response {
    let mut context = Context::new();
    inject_global_vars(&mut context, config, csrf_token);

    match tera.render("500", &context) {
        Ok(html) => (StatusCode::INTERNAL_SERVER_ERROR, Html(html)).into_response(),
        Err(_) => fallback_500_html(),
    }
}

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

/// Injecte variables globales pour templates
fn inject_global_vars(context: &mut Context, config: &RuniqueConfig, csrf_token: Option<String>) {
    context.insert("static_runique", &config.static_files.static_runique_url);
    context.insert("timestamp", &Utc::now().to_rfc3339());
    if let Some(token) = csrf_token {
        context.insert("csrf_token", &token);
    }
    context.insert("debug", &config.debug);
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
