//! HTTP error management middleware: contextual HTML or JSON rendering based on Accept header.
use crate::utils::{
    aliases::{ARuniqueConfig, ATera, StrMap},
    error_key::DEBUG_MESSAGE_KEYS,
};
use crate::{
    config::RuniqueConfig,
    errors::error::{ErrorContext, ErrorType, RuniqueError},
    utils::csrf::CsrfToken,
    utils::trad::t,
};
use axum::{
    extract::Extension,
    http::{HeaderValue, Request, StatusCode, header::HeaderName},
    middleware::Next,
    response::{Html, IntoResponse, Response},
};
use chrono::Utc;
use std::sync::Arc;
use tera::{Context, Tera};
use tracing::{error, info, instrument};
use tracing_futures::Instrument;

/// Transport for request info used in contextual debug
pub struct RequestInfoHelper {
    pub method: String,
    pub path: String,
    pub query: Option<String>,
    pub headers: StrMap,
}

/// Principal Runique middleware with tracing + debug
#[instrument(name = "RuniqueRequest", skip(tera, config, next))]
pub async fn error_handler_middleware(
    Extension(tera): Extension<ATera>,
    Extension(config): Extension<ARuniqueConfig>,
    request: Request<axum::body::Body>,
    next: Next,
) -> Response {
    // --- Collect request info ---
    let csrf_token: Option<String> = request.extensions().get::<CsrfToken>().map(|t| t.0.clone());
    let request_helper = RequestInfoHelper {
        method: request.method().to_string(),
        path: request.uri().path().to_string(),
        query: request.uri().query().map(std::string::ToString::to_string),
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

    // --- Execute request within tracing span ---
    let span = tracing::Span::current();
    let response = next.run(request).instrument(span.clone()).await;

    let status = response.status();

    // --- Error handling ---
    if status.is_server_error()
        || status == StatusCode::NOT_FOUND
        || status == StatusCode::TOO_MANY_REQUESTS
    {
        // 429: direct rendering, no debug page
        if status == StatusCode::TOO_MANY_REQUESTS {
            return render_429(&tera, &config, csrf_token);
        }

        let error_ctx = build_error_context(&response, &request_helper, &tera);

        // --- Render according to debug or production mode ---
        if config.debug {
            return render_debug_error_from_context(&tera, &config, &error_ctx, csrf_token);
        } else {
            return match error_ctx.error_type {
                ErrorType::NotFound => render_404(&tera, &config, csrf_token),
                _ => render_500(&tera, &config, csrf_token),
            };
        }
    }

    // --- No error: return normal response ---
    response
}

/// Builds the `ErrorContext` from the response
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

    // No explicit error, create a basic context
    if response.status() == StatusCode::NOT_FOUND {
        ErrorContext::not_found(&request_helper.path).with_request_helper(request_helper)
    } else {
        ErrorContext::generic(response.status(), &t("error.internal_occurred"))
            .with_request_helper(request_helper)
    }
}

/// Logs Runique error according to its severity
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

fn inject_security_headers(headers: &mut axum::http::HeaderMap) {
    let headers_to_add = [
        (
            "content-security-policy",
            "default-src 'none'; script-src 'self'; style-src 'self'; img-src 'self'; font-src 'self'; connect-src 'self'; frame-ancestors 'none'; base-uri 'self'; form-action 'self';",
        ),
        ("x-content-type-options", "nosniff"),
        ("x-frame-options", "DENY"),
        (
            "strict-transport-security",
            "max-age=31536000; includeSubDomains; preload",
        ),
        ("referrer-policy", "strict-origin-when-cross-origin"),
    ];

    for (key, value) in headers_to_add {
        if let (Ok(name), Ok(val)) = (
            HeaderName::from_bytes(key.as_bytes()),
            HeaderValue::from_str(value),
        ) {
            headers.insert(name, val);
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

    let mut response = match tera.render("404", &context) {
        Ok(html) => (StatusCode::NOT_FOUND, Html(html)).into_response(),
        Err(e) => {
            error!("Failed to render 404 template: {}", e);
            fallback_404_html()
        }
    };

    inject_security_headers(response.headers_mut());
    response
}

fn render_429(tera: &Tera, config: &RuniqueConfig, csrf_token: Option<String>) -> Response {
    let mut context = Context::new();
    inject_global_vars(&mut context, config, csrf_token);
    context.insert("error_title", &t("html.429_title"));
    context.insert("error_text", &t("html.429_text"));
    context.insert("back_home", &t("html.back_home"));

    let mut response = match tera.render("429", &context) {
        Ok(html) => (StatusCode::TOO_MANY_REQUESTS, Html(html)).into_response(),
        Err(_) => fallback_429_html(),
    };
    inject_security_headers(response.headers_mut());
    response
}

fn render_500(tera: &Tera, config: &RuniqueConfig, csrf_token: Option<String>) -> Response {
    let mut context = Context::new();
    inject_global_vars(&mut context, config, csrf_token);
    context.insert("error_title", &t("html.500_title"));
    context.insert("error_text", &t("html.500_text"));
    context.insert("back_home", &t("html.back_home"));

    let mut response = match tera.render("500", &context) {
        Ok(html) => (StatusCode::INTERNAL_SERVER_ERROR, Html(html)).into_response(),
        Err(e) => {
            error!("Failed to render 500 template: {}", e);
            fallback_500_html()
        }
    };
    inject_security_headers(response.headers_mut());
    response
}

fn insert_debug_messages(context: &mut Context) {
    for key in DEBUG_MESSAGE_KEYS {
        let translation_key = format!("TemplateMessage.{key}");
        context.insert(*key, &t(&translation_key));
    }
}

fn render_debug_error_from_context(
    tera: &Tera,
    config: &RuniqueConfig,
    error_ctx: &ErrorContext,
    csrf_token: Option<String>,
) -> Response {
    let mut context = match Context::from_serialize(error_ctx) {
        Ok(ctx) => ctx,
        Err(e) => {
            error!("Failed to serialize error context: {}", e);
            return critical_error_html(&format!("Serialization Error: {e}"));
        }
    };
    inject_global_vars(&mut context, config, csrf_token);
    insert_debug_messages(&mut context);

    let mut response = match tera.render("debug", &context) {
        Ok(html) => (
            StatusCode::from_u16(error_ctx.status_code)
                .unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Html(html),
        )
            .into_response(),
        Err(e) => {
            error!("Failed to render debug template: {}", e);
            critical_error_html(&format!("Tera Rendering Error: {e}"))
        }
    };
    inject_security_headers(response.headers_mut());
    response
}

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

#[cfg(test)]
mod tests {
    use super::html_escape;

    #[test]
    fn test_html_escape_ampersand() {
        assert_eq!(html_escape("a & b"), "a &amp; b");
    }

    #[test]
    fn test_html_escape_lt_gt() {
        assert_eq!(html_escape("<script>"), "&lt;script&gt;");
    }

    #[test]
    fn test_html_escape_double_quote() {
        assert_eq!(html_escape(r#"say "hi""#), "say &quot;hi&quot;");
    }

    #[test]
    fn test_html_escape_single_quote() {
        assert_eq!(html_escape("it's"), "it&#x27;s");
    }

    #[test]
    fn test_html_escape_empty() {
        assert_eq!(html_escape(""), "");
    }

    #[test]
    fn test_html_escape_mixed() {
        assert_eq!(
            html_escape("<b>\"Hello\" & 'World'</b>"),
            "&lt;b&gt;&quot;Hello&quot; &amp; &#x27;World&#x27;&lt;/b&gt;"
        );
    }

    #[test]
    fn test_fallback_404_returns_not_found() {
        let resp = super::fallback_404_html();
        assert_eq!(resp.status(), axum::http::StatusCode::NOT_FOUND);
    }

    #[test]
    fn test_fallback_429_returns_too_many_requests() {
        let resp = super::fallback_429_html();
        assert_eq!(resp.status(), axum::http::StatusCode::TOO_MANY_REQUESTS);
    }

    #[test]
    fn test_fallback_500_returns_internal_server_error() {
        let resp = super::fallback_500_html();
        assert_eq!(resp.status(), axum::http::StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[test]
    fn test_critical_error_html_escapes_input() {
        let resp = super::critical_error_html("<script>alert(1)</script>");
        assert_eq!(resp.status(), axum::http::StatusCode::INTERNAL_SERVER_ERROR);
    }
}
