use crate::utils::{
    aliases::{ARuniqueConfig, ATera, StrMap},
    error::DEBUG_MESSAGE_KEYS,
};
use crate::{
    config::RuniqueConfig,
    errors::error::{ErrorContext, ErrorType, RuniqueError},
    utils::csrf::CsrfToken,
    utils::trad::t,
};
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
    // --- Collecte des infos requête ---
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

    // --- Exécute la requête dans le span tracing ---
    let span = tracing::Span::current();
    let response = next.run(request).instrument(span.clone()).await;

    let status = response.status();

    // --- Gestion des erreurs ---
    if status.is_server_error()
        || status == StatusCode::NOT_FOUND
        || status == StatusCode::TOO_MANY_REQUESTS
    {
        // 429 : rendu direct, pas de debug page
        if status == StatusCode::TOO_MANY_REQUESTS {
            return render_429(&tera, &config, csrf_token);
        }

        let error_ctx = build_error_context(&response, &request_helper, &tera);

        // --- Rendu selon mode debug ou production ---
        if config.debug {
            return render_debug_error_from_context(&tera, &config, error_ctx, csrf_token);
        } else {
            return match error_ctx.error_type {
                ErrorType::NotFound => render_404(&tera, &config, csrf_token),
                _ => render_500(&tera, &config, csrf_token),
            };
        }
    }

    // --- Pas d'erreur : retourne la réponse normale ---
    response
}

/// Construit le ErrorContext depuis la réponse
fn build_error_context(
    response: &Response,
    request_helper: &RequestInfoHelper,
    tera: &Tera,
) -> ErrorContext {
    let error_context_from_app = response.extensions().get::<Arc<ErrorContext>>().cloned();
    let error_runique = response
        .extensions()
        .get::<Arc<RuniqueError>>()
        .map(|err| (**err).clone());

    if let Some(ctx) = error_context_from_app {
        info!(
            method = %request_helper.method,
            path = %request_helper.path,
            error_type = ?ctx.error_type,
            "Error with full context"
        );

        let mut ctx = (*ctx).clone();
        if ctx.request_info.is_none() {
            ctx = ctx.with_request_helper(request_helper);
        }
        return ctx;
    }

    if let Some(err) = error_runique {
        log_runique_error(&err, request_helper);
        return ErrorContext::from_runique_error(
            &err,
            Some(&request_helper.path),
            Some(request_helper),
            None,
            Some(tera),
        );
    }

    // Pas d'erreur explicite, créer un contexte basique
    if response.status() == StatusCode::NOT_FOUND {
        ErrorContext::not_found(&request_helper.path).with_request_helper(request_helper)
    } else {
        ErrorContext::generic(response.status(), &t("error.internal_occurred"))
            .with_request_helper(request_helper)
    }
}

/// Log l'erreur Runique selon sa gravité
fn log_runique_error(err: &RuniqueError, request_helper: &RequestInfoHelper) {
    match err {
        RuniqueError::Internal
        | RuniqueError::Database(_)
        | RuniqueError::Io(_)
        | RuniqueError::Template(_)
        | RuniqueError::Custom { .. }
        | RuniqueError::Build(_) => {
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
}

// --- Render Helpers ---

fn render_404(tera: &Tera, config: &RuniqueConfig, csrf_token: Option<String>) -> Response {
    let mut context = Context::new();
    inject_global_vars(&mut context, config, csrf_token);
    context.insert("error_title", &t("html.404_title"));
    context.insert("error_text", &t("html.404_text"));
    context.insert("back_home", &t("html.back_home"));

    match tera.render("404", &context) {
        Ok(html) => (StatusCode::NOT_FOUND, Html(html)).into_response(),
        Err(e) => {
            error!("Failed to render 404 template: {}", e);
            fallback_404_html()
        }
    }
}

fn render_429(tera: &Tera, config: &RuniqueConfig, csrf_token: Option<String>) -> Response {
    let mut context = Context::new();
    inject_global_vars(&mut context, config, csrf_token);
    context.insert("error_title", &t("html.429_title"));
    context.insert("error_text", &t("html.429_text"));
    context.insert("back_home", &t("html.back_home"));

    match tera.render("429", &context) {
        Ok(html) => (StatusCode::TOO_MANY_REQUESTS, Html(html)).into_response(),
        Err(_) => fallback_429_html(),
    }
}

fn render_500(tera: &Tera, config: &RuniqueConfig, csrf_token: Option<String>) -> Response {
    let mut context = Context::new();
    inject_global_vars(&mut context, config, csrf_token);
    context.insert("error_title", &t("html.500_title"));
    context.insert("error_text", &t("html.500_text"));
    context.insert("back_home", &t("html.back_home"));

    match tera.render("500", &context) {
        Ok(html) => (StatusCode::INTERNAL_SERVER_ERROR, Html(html)).into_response(),
        Err(e) => {
            error!("Failed to render 500 template: {}", e);
            fallback_500_html()
        }
    }
}

/// Insère tous les messages de debug dans le contexte (itératif)
fn insert_debug_messages(context: &mut Context) {
    for key in DEBUG_MESSAGE_KEYS {
        let translation_key = format!("TemplateMessage.{}", key);
        context.insert(*key, &t(&translation_key));
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
    insert_debug_messages(&mut context);

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
    context.insert("lang", &crate::utils::trad::current_lang().code());
}

// --- FALLBACKS ---

fn fallback_404_html() -> Response {
    let lang = crate::utils::trad::current_lang().code();
    let html = format!(
        r#"<!DOCTYPE html>
<html lang="{lang}">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{title} - {text}</title>
    <style>
        body {{
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
            display: flex;
            justify-content: center;
            align-items: center;
            min-height: 100vh;
            margin: 0;
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            color: #fff;
        }}
        .container {{
            text-align: center;
            padding: 2rem;
        }}
        h1 {{
            font-size: 6rem;
            margin: 0;
            font-weight: 700;
        }}
        p {{
            font-size: 1.5rem;
            margin: 1rem 0;
        }}
        a {{
            color: #fff;
            text-decoration: none;
            border: 2px solid #fff;
            padding: 0.75rem 1.5rem;
            border-radius: 0.5rem;
            display: inline-block;
            margin-top: 1rem;
            transition: all 0.3s ease;
        }}
        a:hover {{
            background: #fff;
            color: #667eea;
        }}
    </style>
</head>
<body>
    <div class="container">
        <h1>{title}</h1>
        <p>{text}</p>
        <a href="/">{back}</a>
    </div>
</body>
</html>"#,
        lang = lang,
        title = t("html.404_title"),
        text = t("html.404_text"),
        back = t("html.back_home"),
    );
    (StatusCode::NOT_FOUND, Html(html)).into_response()
}

fn fallback_429_html() -> Response {
    let lang = crate::utils::trad::current_lang().code();
    let html = format!(
        r#"<!DOCTYPE html>
<html lang="{lang}">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{title}</title>
    <style>
        body {{
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
            display: flex;
            justify-content: center;
            align-items: center;
            min-height: 100vh;
            margin: 0;
            background: linear-gradient(135deg, #f7971e 0%, #ffd200 100%);
            color: #fff;
        }}
        .container {{ text-align: center; padding: 2rem; }}
        h1 {{ font-size: 6rem; margin: 0; font-weight: 700; }}
        p {{ font-size: 1.5rem; margin: 1rem 0; }}
        a {{
            color: #fff; text-decoration: none; border: 2px solid #fff;
            padding: 0.75rem 1.5rem; border-radius: 0.5rem;
            display: inline-block; margin-top: 1rem; transition: all 0.3s ease;
        }}
        a:hover {{ background: #fff; color: #f7971e; }}
    </style>
</head>
<body>
    <div class="container">
        <h1>{title}</h1>
        <p>{text}</p>
        <a href="/">{back}</a>
    </div>
</body>
</html>"#,
        lang = lang,
        title = t("html.429_title"),
        text = t("html.429_text"),
        back = t("html.back_home"),
    );
    (StatusCode::TOO_MANY_REQUESTS, Html(html)).into_response()
}

fn fallback_500_html() -> Response {
    let lang = crate::utils::trad::current_lang().code();
    let html = format!(
        r#"<!DOCTYPE html>
<html lang="{lang}">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{title} - {text}</title>
    <style>
        body {{
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
            display: flex;
            justify-content: center;
            align-items: center;
            min-height: 100vh;
            margin: 0;
            background: linear-gradient(135deg, #f093fb 0%, #f5576c 100%);
            color: #fff;
        }}
        .container {{
            text-align: center;
            padding: 2rem;
        }}
        h1 {{
            font-size: 6rem;
            margin: 0;
            font-weight: 700;
        }}
        p {{
            font-size: 1.5rem;
            margin: 1rem 0;
        }}
        a {{
            color: #fff;
            text-decoration: none;
            border: 2px solid #fff;
            padding: 0.75rem 1.5rem;
            border-radius: 0.5rem;
            display: inline-block;
            margin-top: 1rem;
            transition: all 0.3s ease;
        }}
        a:hover {{
            background: #fff;
            color: #f5576c;
        }}
    </style>
</head>
<body>
    <div class="container">
        <h1>{title}</h1>
        <p>{text}</p>
        <p style="font-size: 1rem;">{notice}</p>
        <a href="/">{back}</a>
    </div>
</body>
</html>"#,
        lang = lang,
        title = t("html.500_title"),
        text = t("html.500_text"),
        notice = t("html.500_notice"),
        back = t("html.back_home"),
    );
    (StatusCode::INTERNAL_SERVER_ERROR, Html(html)).into_response()
}

fn critical_error_html(error: &str) -> Response {
    let lang = crate::utils::trad::current_lang().code();
    let html = format!(
        r#"<!DOCTYPE html>
<html lang="{lang}">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{title_tag}</title>
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
        <h1>⚠️ {title} ⚠️</h1>
        <p>{text}</p>
        <p>{contact}</p>
        <pre>{error}</pre>
    </div>
</body>
</html>"#,
        lang = lang,
        title_tag = t("html.critical_error_title"),
        title = t("html.critical_error_title"),
        text = t("html.critical_error_text"),
        contact = t("html.critical_error_contact"),
        error = html_escape(error),
    );
    (StatusCode::INTERNAL_SERVER_ERROR, Html(html)).into_response()
}

pub(crate) fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#x27;")
}
