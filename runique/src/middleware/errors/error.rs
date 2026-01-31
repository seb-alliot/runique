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
    errors::error::{ErrorContext, ErrorType, RuniqueError},
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
        // ✅ Essaie de récupérer SOIT un ErrorContext (venant de AppError)
        //    SOIT un RuniqueError (venant de handlers qui retournent RuniqueError)
        let error_context_from_app = response.extensions().get::<Arc<ErrorContext>>().cloned();

        let error_runique = response
            .extensions()
            .get::<Arc<RuniqueError>>()
            .map(|err| (**err).clone());

        let error_ctx = if let Some(ctx) = error_context_from_app {
            // ✅ On a déjà un ErrorContext complet (venant de AppError)
            info!(
                method = %request_helper.method,
                path = %request_helper.path,
                error_type = ?ctx.error_type,
                "Error with full context"
            );

            // Enrichir avec les infos de requête si pas déjà présent
            let mut ctx = (*ctx).clone();
            if ctx.request_info.is_none() {
                ctx = ctx.with_request_helper(&request_helper);
            }
            ctx
        } else if let Some(err) = error_runique {
            // ✅ On a un RuniqueError (ancien système)
            match &err {
                RuniqueError::Internal
                | RuniqueError::Database(_)
                | RuniqueError::Io(_)
                | RuniqueError::Template(_)
                | RuniqueError::Custom { .. } => {
                    error!(
                        method = %request_helper.method,
                        path = %request_helper.path,
                        error = %err,
                        "Critical error occurred"
                    );
                }
                RuniqueError::Validation(_) | RuniqueError::Forbidden | RuniqueError::NotFound => {
                    info!(
                        method = %request_helper.method,
                        path = %request_helper.path,
                        error = %err,
                        "Handled error"
                    );
                }
            }

            // Créer un contexte enrichi depuis RuniqueError
            ErrorContext::from_runique_error(
                &err,
                Some(&request_helper.path),
                Some(&request_helper),
                None,        // nom du template si applicable
                Some(&tera), // tera pour template_info si template error
            )
        } else {
            // ✅ Pas d'erreur explicite, créer un contexte basique
            if status == StatusCode::NOT_FOUND {
                ErrorContext::not_found(&request_helper.path).with_request_helper(&request_helper)
            } else {
                ErrorContext::generic(status, "Une erreur est survenue")
                    .with_request_helper(&request_helper)
            }
        };

        // ---  Rendu selon mode debug ou production ---
        if config.debug {
            return render_debug_error_from_context(&tera, &config, error_ctx, csrf_token);
        } else {
            // Production : 404 ou 500 simple selon le type d'erreur
            return match error_ctx.error_type {
                ErrorType::NotFound => render_404(&tera, &config, csrf_token),
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
        Err(e) => {
            error!("Failed to render 404 template: {}", e);
            fallback_404_html()
        }
    }
}

fn render_500(tera: &Tera, config: &RuniqueConfig, csrf_token: Option<String>) -> Response {
    let mut context = Context::new();
    inject_global_vars(&mut context, config, csrf_token);

    match tera.render("500", &context) {
        Ok(html) => (StatusCode::INTERNAL_SERVER_ERROR, Html(html)).into_response(),
        Err(e) => {
            error!("Failed to render 500 template: {}", e);
            fallback_500_html()
        }
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
        Err(e) => {
            error!("Failed to serialize error context: {}", e);
            return critical_error_html(&format!("Serialization Error: {}", e));
        }
    };

    inject_global_vars(&mut context, config, csrf_token);

    match tera.render("debug", &context) {
        Ok(html) => (
            StatusCode::from_u16(error_ctx.status_code)
                .unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Html(html),
        )
            .into_response(),
        Err(e) => {
            error!("Failed to render debug template: {}", e);
            critical_error_html(&format!("Tera Rendering Error: {}", e))
        }
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
    let html = r#"<!DOCTYPE html>
<html lang="fr">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>404 - Page non trouvée</title>
    <style>
        body {
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
            display: flex;
            justify-content: center;
            align-items: center;
            min-height: 100vh;
            margin: 0;
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            color: #fff;
        }
        .container {
            text-align: center;
            padding: 2rem;
        }
        h1 {
            font-size: 6rem;
            margin: 0;
            font-weight: 700;
        }
        p {
            font-size: 1.5rem;
            margin: 1rem 0;
        }
        a {
            color: #fff;
            text-decoration: none;
            border: 2px solid #fff;
            padding: 0.75rem 1.5rem;
            border-radius: 0.5rem;
            display: inline-block;
            margin-top: 1rem;
            transition: all 0.3s ease;
        }
        a:hover {
            background: #fff;
            color: #667eea;
        }
    </style>
</head>
<body>
    <div class="container">
        <h1>404</h1>
        <p>Page non trouvée</p>
        <a href="/">Retour à l'accueil</a>
    </div>
</body>
</html>"#;
    (StatusCode::NOT_FOUND, Html(html)).into_response()
}

fn fallback_500_html() -> Response {
    let html = r#"<!DOCTYPE html>
<html lang="fr">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>500 - Erreur serveur</title>
    <style>
        body {
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
            display: flex;
            justify-content: center;
            align-items: center;
            min-height: 100vh;
            margin: 0;
            background: linear-gradient(135deg, #f093fb 0%, #f5576c 100%);
            color: #fff;
        }
        .container {
            text-align: center;
            padding: 2rem;
        }
        h1 {
            font-size: 6rem;
            margin: 0;
            font-weight: 700;
        }
        p {
            font-size: 1.5rem;
            margin: 1rem 0;
        }
        a {
            color: #fff;
            text-decoration: none;
            border: 2px solid #fff;
            padding: 0.75rem 1.5rem;
            border-radius: 0.5rem;
            display: inline-block;
            margin-top: 1rem;
            transition: all 0.3s ease;
        }
        a:hover {
            background: #fff;
            color: #f5576c;
        }
    </style>
</head>
<body>
    <div class="container">
        <h1>500</h1>
        <p>Erreur serveur interne</p>
        <p style="font-size: 1rem;">Nos équipes ont été notifiées et travaillent sur le problème.</p>
        <a href="/">Retour à l'accueil</a>
    </div>
</body>
</html>"#;
    (StatusCode::INTERNAL_SERVER_ERROR, Html(html)).into_response()
}

fn critical_error_html(error: &str) -> Response {
    let html = format!(
        r#"<!DOCTYPE html>
<html lang="fr">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Erreur critique</title>
    <style>
        body {{
            font-family: 'Courier New', monospace;
            padding: 2rem;
            background: #1a1a1a;
            color: #00ff00;
            margin: 0;
        }}
        .container {{
            max-width: 1200px;
            margin: 0 auto;
            background: #000;
            border: 2px solid #00ff00;
            padding: 2rem;
            border-radius: 0.5rem;
        }}
        h1 {{
            color: #ff0000;
            margin: 0 0 1rem 0;
            font-size: 2rem;
            text-shadow: 0 0 10px #ff0000;
        }}
        p {{
            margin: 1rem 0;
            line-height: 1.6;
        }}
        pre {{
            background: #0a0a0a;
            padding: 1rem;
            border: 1px solid #00ff00;
            border-radius: 0.25rem;
            overflow-x: auto;
            color: #ff6b6b;
            white-space: pre-wrap;
            word-wrap: break-word;
        }}
    </style>
</head>
<body>
    <div class="container">
        <h1>⚠️ CRITICAL ERROR ⚠️</h1>
        <p>Le système de gestion d'erreurs a lui-même rencontré une erreur.</p>
        <p>Cette situation ne devrait jamais se produire. Veuillez contacter l'administrateur système.</p>
        <pre>{}</pre>
    </div>
</body>
</html>"#,
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
