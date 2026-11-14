//! HTTP error management middleware: renders the HTML error pages (`{code}.html`, the debug page).
use crate::utils::{
    aliases::{ARuniqueConfig, ATera, StrMap},
    error_key::DEBUG_MESSAGE_KEYS,
};
use crate::{
    config::RuniqueConfig,
    errors::error::{ErrorContext, ErrorType, RuniqueError, app_package},
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
use tracing::{Level, error, instrument};
use tracing_futures::Instrument;

/// Level for non-critical HTTP error diagnostics (status/path of handled errors).
/// `None` (disabled) by default — opt-in observability via `errors.http`.
fn errors_http_level() -> Option<Level> {
    crate::utils::runique_log::log_level!(errors.http)
}

/// Level for error-template render failures. Floored at `WARN`: a broken error
/// template still serves a fallback, but the dev must see it. Configurable via
/// `errors.render`.
fn errors_render_level() -> Level {
    crate::utils::runique_log::log_level!(errors.render).unwrap_or(Level::WARN)
}

/// A header whose name holds one of these is left off the debug page: such a
/// page ends up in screenshots, shared screens and issues. `auth` also covers
/// `authorization` and `authenticate`.
const SENSITIVE_HEADER_WORDS: &[&str] = &[
    "auth",
    "cookie",
    "session",
    "token",
    "key",
    "secret",
    "password",
    "passwd",
    "credential",
    "signature",
];

fn redact_sensitive_path(path: &str) -> String {
    let mut parts: Vec<&str> = path.split('/').collect();
    if let Some(index) = parts.iter().position(|part| {
        let part = part.to_ascii_lowercase();
        ["reset", "token", "password", "forgot", "invite", "verify"]
            .iter()
            .any(|marker| part.contains(marker))
    }) && let Some(secret) = parts.get_mut(index + 1)
    {
        *secret = "[redacted]";
    }
    parts.join("/")
}

/// Transport for request info used in contextual debug
pub struct RequestInfoHelper {
    pub method: String,
    pub path: String,
    pub query: Option<String>,
    pub headers: StrMap,
}

/// Principal Runique middleware with tracing + debug.
///
/// The span records only `method` + `uri` — never the full `request` Debug, which
/// would dump every header onto every child log line and drown the console.
#[instrument(
    name = "RuniqueRequest",
    skip(tera, config, next, request),
    fields(method = %request.method(), uri = %redact_sensitive_path(request.uri().path()))
)]
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
        path: redact_sensitive_path(request.uri().path()),
        query: None,
        headers: request
            .headers()
            .iter()
            .filter(|(k, _)| {
                let name = k.as_str().to_lowercase();
                !SENSITIVE_HEADER_WORDS
                    .iter()
                    .any(|word| name.contains(word))
            })
            .map(|(k, v)| (k.to_string(), v.to_str().unwrap_or("").to_string()))
            .collect(),
    };

    // --- Execute request within tracing span ---
    let span = tracing::Span::current();
    let response = next.run(request).instrument(span.clone()).await;

    let status = response.status();

    // Only the 400s Runique built itself (`ErrorType::BadRequest`, e.g. a bad
    // query string): an API's own 400 body is left untouched.
    let framework_bad_request = status == StatusCode::BAD_REQUEST
        && response
            .extensions()
            .get::<Arc<ErrorContext>>()
            .is_some_and(|ctx| matches!(ctx.error_type, ErrorType::BadRequest));

    // A 413 Runique built (an upload or a form over its limit): direct
    // rendering, like the 429. The debug page would add nothing to it.
    if status == StatusCode::PAYLOAD_TOO_LARGE
        && let Some(FrameworkTooLarge(reason)) = response.extensions().get::<FrameworkTooLarge>()
    {
        return render_status_page(&tera, &config, csrf_token, status, Some(reason.as_str()));
    }

    // --- Error handling ---
    if status.is_server_error()
        || status == StatusCode::NOT_FOUND
        || status == StatusCode::TOO_MANY_REQUESTS
        || framework_bad_request
    {
        // 429: direct rendering, no debug page. The wait the limiter advertised
        // stays on the page: this replacement would otherwise drop it.
        if status == StatusCode::TOO_MANY_REQUESTS {
            let retry_after = response
                .headers()
                .get(axum::http::header::RETRY_AFTER)
                .cloned();
            let mut page = render_status_page(&tera, &config, csrf_token, status, None);
            if let Some(value) = retry_after {
                page.headers_mut()
                    .insert(axum::http::header::RETRY_AFTER, value);
            }
            return page;
        }

        // 503: capacity/overload — direct rendering with Retry-After, no debug page
        if status == StatusCode::SERVICE_UNAVAILABLE {
            let mut page = render_status_page(&tera, &config, csrf_token, status, None);
            page.headers_mut().insert(
                axum::http::header::RETRY_AFTER,
                HeaderValue::from(SERVICE_UNAVAILABLE_RETRY_AFTER_SECS),
            );
            return page;
        }

        let error_ctx = build_error_context(&response, &request_helper, &tera);

        // --- Render according to debug or production mode ---
        if config.debug {
            return render_debug_error_from_context(&tera, &config, &error_ctx, csrf_token);
        } else {
            return match error_ctx.error_type {
                ErrorType::NotFound => {
                    render_status_page(&tera, &config, csrf_token, StatusCode::NOT_FOUND, None)
                }
                ErrorType::BadRequest => {
                    render_status_page(&tera, &config, csrf_token, StatusCode::BAD_REQUEST, None)
                }
                _ => render_status_page(
                    &tera,
                    &config,
                    csrf_token,
                    StatusCode::INTERNAL_SERVER_ERROR,
                    None,
                ),
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
        if let Some(level) = errors_http_level() {
            crate::runique_log!(
                level,
                method = %request_helper.method,
                path = %request_helper.path,
                error_type = ?ctx.error_type,
                "Error with full context"
            );
        }

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
        | RuniqueError::Template { .. }
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
            if let Some(level) = errors_http_level() {
                crate::runique_log!(
                    level,
                    method = %request_helper.method,
                    path = %request_helper.path,
                    error = %err,
                    "Handled error"
                );
            }
        }
    }
}

fn inject_security_headers(headers: &mut axum::http::HeaderMap, config: &RuniqueConfig) {
    let headers_to_add = [
        (
            "content-security-policy",
            "default-src 'none'; script-src 'self'; style-src 'self'; img-src 'self'; font-src 'self'; connect-src 'self'; frame-ancestors 'none'; base-uri 'self'; form-action 'self';",
        ),
        ("x-content-type-options", "nosniff"),
        ("x-frame-options", "DENY"),
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

    // HSTS is gated and configurable (same source as the rest of the framework).
    if let Some(hsts) = config.security.hsts_header_value()
        && let Ok(val) = HeaderValue::from_str(&hsts)
    {
        headers.insert(HeaderName::from_static("strict-transport-security"), val);
    }
}

// --- Render Helpers ---

/// Marks a 413 Runique built itself (a body over one of its limits), with
/// its translated reason: only those get `413.html`, which shows the reason;
/// an API's own 413 keeps its body.
#[derive(Clone)]
pub(crate) struct FrameworkTooLarge(String);

/// A 413 built by Runique: `message` as the body for an HTTP client, and as
/// the text of the page `error_handler_middleware` renders for a browser.
pub(crate) fn too_large(message: String) -> Response {
    let mut res = (StatusCode::PAYLOAD_TOO_LARGE, message.clone()).into_response();
    res.extensions_mut().insert(FrameworkTooLarge(message));
    res
}

/// Retry delay (seconds) advertised on a 503 so clients back off before retrying.
const SERVICE_UNAVAILABLE_RETRY_AFTER_SECS: u32 = 30;

fn status_icon(status: StatusCode) -> &'static str {
    match status {
        StatusCode::NOT_FOUND => "🔍",
        StatusCode::TOO_MANY_REQUESTS | StatusCode::SERVICE_UNAVAILABLE => "⏳",
        _ => "⚠️",
    }
}

/// The page of a status: `{code}.html`, with the `html.{code}_title` text and
/// `text` (by default `html.{code}_text`) in the visitor's language. A
/// project replaces one status's page by declaring its own `{code}.html`.
fn render_status_page(
    tera: &Tera,
    config: &RuniqueConfig,
    csrf_token: Option<String>,
    status: StatusCode,
    text: Option<&str>,
) -> Response {
    let code = status.as_u16();
    let text = text.map_or_else(
        || t(&format!("html.{code}_text")),
        |text| text.to_owned().into(),
    );
    let mut context = Context::new();
    inject_global_vars(&mut context, config, csrf_token);
    context.insert("error_title", &t(&format!("html.{code}_title")));
    context.insert("error_text", &text);
    context.insert("error_icon", status_icon(status));
    context.insert("back_home", &t("html.back_home"));

    let template = format!("{code}.html");
    let mut response = match tera.render(&template, &context) {
        Ok(html) => (status, Html(html)).into_response(),
        Err(e) => {
            crate::runique_log!(errors_render_level(), error = %e, template = %template, "failed to render error template");
            fallback_status_html(status, &text)
        }
    };
    inject_security_headers(response.headers_mut(), config);
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
    let (app_name, app_version) = app_package(&config.base_dir);
    context.insert("app_name", &app_name);
    context.insert("app_version", &app_version);

    let mut response = match tera.render("debug.html", &context) {
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
    inject_security_headers(response.headers_mut(), config);
    response
}

fn inject_global_vars(context: &mut Context, config: &RuniqueConfig, csrf_token: Option<String>) {
    context.insert("static_runique", &config.static_files.static_runique_url);
    context.insert(
        "timestamp",
        &Utc::now().format("%Y-%m-%d %H:%M:%S UTC").to_string(),
    );
    if let Some(token) = csrf_token {
        context.insert("csrf_token", &token);
    }
    context.insert("debug", &config.debug);
    context.insert("dev", &config.dev);
    context.insert("lang", &crate::utils::trad::current_lang().code());
}

// --- FALLBACKS ---

/// The page of a status when its template can't be rendered: no style, no
/// script, the text escaped.
fn fallback_status_html(status: StatusCode, text: &str) -> Response {
    let code = status.as_u16();
    let html = format!(
        "<!DOCTYPE html><html lang=\"{lang}\"><head><meta charset=\"UTF-8\"><title>{code}</title></head><body><h1>{code}</h1><p>{text}</p><a href=\"/\">{back}</a></body></html>",
        lang = crate::utils::trad::current_lang().code(),
        text = html_escape(text),
        back = html_escape(&t("html.back_home")),
    );
    (status, Html(html)).into_response()
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
    use super::{errors_http_level, errors_render_level, html_escape, redact_sensitive_path};
    use crate::utils::runique_log::{RuniqueLog, log_init, reset_log_for_test};
    use tracing::Level;

    #[test]
    #[serial_test::serial]
    fn errors_render_floors_at_warn_and_http_off_by_default() {
        reset_log_for_test();
        log_init(RuniqueLog::new());
        // render must stay visible (WARN) even with the node disabled; http is opt-in.
        assert_eq!(errors_render_level(), Level::WARN);
        assert!(errors_http_level().is_none());
        reset_log_for_test();
    }

    #[test]
    #[serial_test::serial]
    fn errors_levels_follow_explicit_config() {
        reset_log_for_test();
        log_init(RuniqueLog::new().errors(|e| e.http(Level::INFO).render(Level::ERROR)));
        assert_eq!(errors_http_level(), Some(Level::INFO));
        assert_eq!(errors_render_level(), Level::ERROR);
        reset_log_for_test();
    }

    #[test]
    fn test_html_escape_ampersand() {
        assert_eq!(html_escape("a & b"), "a &amp; b");
    }

    #[test]
    fn test_html_escape_lt_gt() {
        assert_eq!(html_escape("<script>"), "&lt;script&gt;");
    }

    #[test]
    fn reset_token_paths_are_redacted_before_logging_or_rendering() {
        let token = "reset-bearer-secret-123";
        let path = redact_sensitive_path(&format!("/reset/{token}"));
        assert_eq!(path, "/reset/[redacted]");
        assert!(!path.contains(token));
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
    fn the_fallback_page_keeps_its_status_and_escapes_its_text() {
        use axum::http::StatusCode;
        for status in [
            StatusCode::BAD_REQUEST,
            StatusCode::NOT_FOUND,
            StatusCode::PAYLOAD_TOO_LARGE,
            StatusCode::TOO_MANY_REQUESTS,
            StatusCode::INTERNAL_SERVER_ERROR,
            StatusCode::SERVICE_UNAVAILABLE,
        ] {
            assert_eq!(super::fallback_status_html(status, "x").status(), status);
        }
    }

    #[tokio::test]
    async fn the_fallback_page_text_is_escaped() {
        use http_body_util::BodyExt;
        let resp = super::fallback_status_html(
            axum::http::StatusCode::NOT_FOUND,
            "<script>alert(1)</script>",
        );
        let body = resp.into_body().collect().await.unwrap().to_bytes();
        let html = String::from_utf8_lossy(&body);
        assert!(!html.contains("<script>"), "{html}");
        assert!(html.contains("&lt;script&gt;"), "{html}");
    }

    #[test]
    fn test_critical_error_html_escapes_input() {
        let resp = super::critical_error_html("<script>alert(1)</script>");
        assert_eq!(resp.status(), axum::http::StatusCode::INTERNAL_SERVER_ERROR);
    }
}
