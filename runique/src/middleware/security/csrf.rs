//! CSRF Middleware: generates and stores the token in session, validates mutating requests.
use crate::auth::session::is_authenticated;
use crate::context::RequestExtensions;
use crate::utils::{
    aliases::AEngine,
    constante::{session::CSRF_TOKEN_KEY, session_key::session::SESSION_USER_ID_KEY},
    csrf::{CsrfContext, CsrfToken},
};
use axum::{
    body::Body,
    extract::State,
    http::{HeaderValue, Method, Request, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
};
use subtle::ConstantTimeEq;
use tera::{Function, Kwargs, TeraResult, Value, escape_html};
use tower_sessions::Session;

pub struct CsrfTokenFunction;

// `tera::State` is aliased locally: `axum::extract::State` is already imported above.
impl Function<TeraResult<Value>> for CsrfTokenFunction {
    fn is_safe(&self) -> bool {
        true
    }

    fn call(&self, kwargs: Kwargs, _: &tera::State) -> TeraResult<Value> {
        let token = kwargs.get::<&str>(CSRF_TOKEN_KEY)?.unwrap_or("");

        // Escaped even though the generator only emits attribute-safe characters:
        // the tag is emitted unescaped, so it must not depend on a guarantee made
        // elsewhere. Mirrors the `csrf_field` filter.
        let mut escaped = Vec::new();
        escape_html(token, &mut escaped)
            .map_err(|e| tera::Error::chain("csrf_token: failed to escape the token", e))?;
        let token = String::from_utf8(escaped)?;

        Ok(Value::safe_string(&format!(
            r#"<input type="hidden" name="csrf_token" value="{token}">"#
        )))
    }
}

pub async fn csrf_middleware(
    State(engine): State<AEngine>,
    session: Session,
    mut req: Request<Body>,
    next: Next,
) -> Response {
    // Strip csrf_token from GET query params to avoid token exposure in URLs
    if matches!(req.method(), &Method::GET | &Method::HEAD) {
        let uri = req.uri();
        if let Some(query) = uri.query()
            && query.split('&').any(|p| p.starts_with("csrf_token="))
        {
            let clean_query: String = query
                .split('&')
                .filter(|p| !p.starts_with("csrf_token="))
                .collect::<Vec<_>>()
                .join("&");

            let new_uri = if clean_query.is_empty() {
                uri.path().to_string()
            } else {
                format!("{}?{}", uri.path(), clean_query)
            };

            if let Ok(location) = HeaderValue::from_str(&new_uri) {
                let mut res = (StatusCode::FOUND, "").into_response();
                res.headers_mut()
                    .insert(axum::http::header::LOCATION, location);
                return res;
            }
        }
    }

    // Skip CSRF for exempt paths (webhooks with their own signature verification)
    if engine
        .csrf_exempt_paths
        .iter()
        .any(|p| p == req.uri().path())
    {
        return next.run(req).await;
    }

    let secret = &engine.config.server.secret_key;

    // Retrieve or generate the session token
    let session_token: CsrfToken = if let Some(t) = session
        .get::<CsrfToken>(CSRF_TOKEN_KEY)
        .await
        .ok()
        .flatten()
    {
        if session.insert(CSRF_TOKEN_KEY, &t).await.is_err() {
            return (StatusCode::INTERNAL_SERVER_ERROR, "Session write error").into_response();
        }
        t
    } else {
        let token = if is_authenticated(&session).await {
            let user_id: crate::utils::pk::Pk = session
                .get::<crate::utils::pk::Pk>(SESSION_USER_ID_KEY)
                .await
                .ok()
                .flatten()
                .unwrap_or(0);
            CsrfToken::generate_with_context(&CsrfContext::Authenticated { user_id }, secret)
        } else {
            let session_id = session.id().map(|id| id.to_string()).unwrap_or_default();
            CsrfToken::generate_with_context(
                &CsrfContext::Anonymous {
                    session_id: &session_id,
                },
                secret,
            )
        };
        if session.insert(CSRF_TOKEN_KEY, &token).await.is_err() {
            return (StatusCode::INTERNAL_SERVER_ERROR, "Session write error").into_response();
        }
        token
    };

    // CSRF verification **ONLY for AJAX requests with header**
    let requires_csrf = matches!(
        req.method(),
        &Method::POST | &Method::PUT | &Method::DELETE | &Method::PATCH
    );

    if requires_csrf {
        let has_header = req.headers().contains_key("X-CSRF-Token");

        // If header present, we validate (AJAX request)
        if has_header {
            let header_token = req
                .headers()
                .get("X-CSRF-Token")
                .and_then(|h| h.to_str().ok())
                .and_then(|masked| CsrfToken::unmasked(masked).ok());

            match header_token {
                Some(token)
                    if token
                        .as_str()
                        .as_bytes()
                        // ct_eq: constant-time comparison — prevents an attacker
                        // from guessing the token byte by byte via response time
                        .ct_eq(session_token.as_str().as_bytes())
                        .into() =>
                {
                    // OK, continue
                }
                _ => {
                    return (StatusCode::FORBIDDEN, "Invalid CSRF token").into_response();
                }
            }
        } else {
            // No CSRF header: allowed only for HTML form submissions
            // (urlencoded / multipart). JSON requests without header are blocked.
            let ct = req
                .headers()
                .get("content-type")
                .and_then(|v| v.to_str().ok())
                .unwrap_or("");
            let is_form = ct.starts_with("application/x-www-form-urlencoded")
                || ct.starts_with("multipart/form-data");
            if !is_form {
                return (StatusCode::FORBIDDEN, "CSRF token required").into_response();
            }
            // Otherwise, we let Prisme validate the form field
        }
    }

    // Inject the raw session token for the request pipeline (form() validation).
    let extensions = RequestExtensions::new().with_csrf_token(session_token.clone());
    extensions.inject_request(&mut req);

    let mut res = next.run(req).await;

    // Expose the token to the frontend, masked per-response (BREACH mitigation).
    // If masking ever fails (would only happen on a non-hex token, which the
    // generator never produces), skip the header rather than leaking the raw
    // unmasked session token in the response.
    if let Ok(masked) = session_token.masked()
        && let Ok(hv) = HeaderValue::from_str(masked.as_str())
    {
        res.headers_mut().insert("X-CSRF-Token", hv);
    }

    res
}
