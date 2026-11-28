//! HTTP error management middleware: renders the HTML error pages (`{code}.html`, the debug page).
use crate::security::escape::html_escape;
use crate::utils::aliases::AEngine;
use crate::{
    config::RuniqueConfig,
    errors::constants::DEBUG_MESSAGE_KEYS,
    errors::{
        ErrorContext, ErrorType, FrameworkTooLarge, RequestSnapshot, RuniqueError, app_package,
    },
    security::csrf::CsrfToken,
    translation::t,
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
    crate::log::log_level!(errors.http)
}

/// Level for error-template render failures. Floored at `WARN`: a broken error
/// template still serves a fallback, but the dev must see it. Configurable via
/// `errors.render`.
fn errors_render_level() -> Level {
    crate::log::log_level!(errors.render).unwrap_or(Level::WARN)
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

/// Principal Runique middleware with tracing + debug.
///
/// The span records only `method` + `uri` — never the full `request` Debug, which
/// would dump every header onto every child log line and drown the console.
#[instrument(
    name = "RuniqueRequest",
    skip(engine, next, request),
    fields(method = %request.method(), uri = %redact_sensitive_path(request.uri().path()))
)]
pub async fn error_handler_middleware(
    Extension(engine): Extension<AEngine>,
    request: Request<axum::body::Body>,
    next: Next,
) -> Response {
    let (tera, config) = (engine.tera.clone(), engine.config.clone());
    // --- Collect request info ---
    let csrf_token: Option<String> = request.extensions().get::<CsrfToken>().map(|t| t.0.clone());
    let request_snapshot = RequestSnapshot {
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

        let error_ctx = build_error_context(&response, &request_snapshot, &tera);

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
    request_snapshot: &RequestSnapshot,
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
                method = %request_snapshot.method,
                path = %request_snapshot.path,
                error_type = ?ctx.error_type,
                "Error with full context"
            );
        }

        let mut ctx = (*ctx).clone();
        if ctx.request_info.is_none() {
            ctx = ctx.with_request_snapshot(request_snapshot);
        }
        return ctx;
    }

    if let Some(err) = error_runique {
        log_runique_error(&err, request_snapshot);
        return ErrorContext::from_runique_error(
            &err,
            Some(&request_snapshot.path),
            Some(request_snapshot),
            None,
            Some(tera),
        );
    }

    // No explicit error, create a basic context
    if response.status() == StatusCode::NOT_FOUND {
        ErrorContext::not_found(&request_snapshot.path).with_request_snapshot(request_snapshot)
    } else {
        ErrorContext::generic(
            response.status(),
            &t(runique_i18n::keys::app::error::INTERNAL_OCCURRED),
        )
        .with_request_snapshot(request_snapshot)
    }
}

/// Logs Runique error according to its severity
fn log_runique_error(err: &RuniqueError, request_snapshot: &RequestSnapshot) {
    match err {
        RuniqueError::Internal
        | RuniqueError::Database(_)
        | RuniqueError::Io(_)
        | RuniqueError::Template { .. }
        | RuniqueError::Custom { .. }
        | RuniqueError::Build(_) => {
            error!(
                method = %request_snapshot.method,
                path = %request_snapshot.path,
                error = %err,
                "Critical error occurred"
            );
        }
        RuniqueError::Validation(_) | RuniqueError::Forbidden | RuniqueError::NotFound => {
            if let Some(level) = errors_http_level() {
                crate::runique_log!(
                    level,
                    method = %request_snapshot.method,
                    path = %request_snapshot.path,
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

/// Retry delay (seconds) advertised on a 503 so clients back off before retrying.
const SERVICE_UNAVAILABLE_RETRY_AFTER_SECS: u32 = 30;

fn status_icon(status: StatusCode) -> &'static str {
    match status {
        StatusCode::NOT_FOUND => "🔍",
        StatusCode::TOO_MANY_REQUESTS | StatusCode::SERVICE_UNAVAILABLE => "⏳",
        _ => "⚠️",
    }
}

/// The title and the default text of the page of a status; `None` for a
/// status Runique has no message for.
fn status_messages(code: u16) -> Option<(&'static str, &'static str)> {
    use runique_i18n::keys::http::{error, label};
    Some(match code {
        400 => (label::CODE_400, error::CODE_400),
        404 => (label::CODE_404, error::CODE_404),
        413 => (label::CODE_413, error::CODE_413),
        429 => (label::CODE_429, error::CODE_429),
        500 => (label::CODE_500, error::CODE_500),
        503 => (label::CODE_503, error::CODE_503),
        _ => return None,
    })
}

/// The page of a status: `{code}.html`, with its title and `text` (by default
/// the status's own message) in the visitor's language; a status without a
/// message of its own shows its code and its standard name. A project
/// replaces one status's page by declaring its own `{code}.html`.
fn render_status_page(
    tera: &Tera,
    config: &RuniqueConfig,
    csrf_token: Option<String>,
    status: StatusCode,
    text: Option<&str>,
) -> Response {
    let code = status.as_u16();
    let (title, default_text): (
        std::borrow::Cow<'static, str>,
        std::borrow::Cow<'static, str>,
    ) = match status_messages(code) {
        Some((title, text)) => (t(title), t(text)),
        None => (
            code.to_string().into(),
            status.canonical_reason().unwrap_or_default().into(),
        ),
    };
    let text = text.map_or(default_text, |text| text.to_owned().into());
    let mut context = Context::new();
    inject_global_vars(&mut context, config, csrf_token);
    context.insert("error_title", &title);
    context.insert("error_text", &text);
    context.insert("error_icon", status_icon(status));
    context.insert("back_home", &t(runique_i18n::keys::http::label::BACK_HOME));

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
    for (variable, key) in DEBUG_MESSAGE_KEYS {
        context.insert(*variable, &t(key));
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
    context.insert("lang", &crate::translation::current_lang().code());
}

// --- FALLBACKS ---

/// The page of a status when its template can't be rendered: no style, no
/// script, the text escaped.
fn fallback_status_html(status: StatusCode, text: &str) -> Response {
    let code = status.as_u16();
    let html = format!(
        "<!DOCTYPE html><html lang=\"{lang}\"><head><meta charset=\"UTF-8\"><title>{code}</title></head><body><h1>{code}</h1><p>{text}</p><a href=\"/\">{back}</a></body></html>",
        lang = crate::translation::current_lang().code(),
        text = html_escape(text),
        back = html_escape(&t(runique_i18n::keys::http::label::BACK_HOME)),
    );
    (status, Html(html)).into_response()
}

fn critical_error_html(error: &str) -> Response {
    let lang = crate::translation::current_lang().code();
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
        title_tag = t(runique_i18n::keys::http::label::CRITICAL),
        title = t(runique_i18n::keys::http::label::CRITICAL),
        text = t(runique_i18n::keys::http::error::CRITICAL_TEXT),
        contact = t(runique_i18n::keys::http::error::CRITICAL_CONTACT),
        error = html_escape(error),
    );
    (StatusCode::INTERNAL_SERVER_ERROR, Html(html)).into_response()
}

#[cfg(test)]
mod tests {
    use super::{errors_http_level, errors_render_level, redact_sensitive_path};
    use crate::log::{RuniqueLog, log_init, reset_log_for_test};
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
    fn reset_token_paths_are_redacted_before_logging_or_rendering() {
        let token = "reset-bearer-secret-123";
        let path = redact_sensitive_path(&format!("/reset/{token}"));
        assert_eq!(path, "/reset/[redacted]");
        assert!(!path.contains(token));
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

#[cfg(test)]
mod status_messages_tests {
    use super::status_messages;
    use crate::translation::t;

    // Each status Runique has a page for has its title and its text, both
    // of that status, and both exist.
    #[test]
    fn each_status_has_its_title_and_its_text() {
        for code in [400u16, 404, 413, 429, 500, 503] {
            let (title, text) = status_messages(code).unwrap_or_else(|| panic!("{code}: none"));
            assert_eq!(title, format!("http.label.{code}"));
            assert_eq!(text, format!("http.error.{code}"));
            assert_ne!(t(title), title, "{code}: no title");
            assert_ne!(t(text), text, "{code}: no text");
        }
    }

    // Any other status has none: its page shows the code, never a key.
    #[test]
    fn a_status_without_a_page_has_no_message() {
        for code in [
            0u16, 200, 301, 401, 403, 405, 418, 422, 499, 501, 502, 504, 999,
        ] {
            assert_eq!(status_messages(code), None, "{code}");
        }
    }
}
