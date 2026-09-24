//! Prisme: non-generic CSRF + body extractor integrated into the Request pipeline.
use crate::forms::prisme::{aegis, sentinel};
use crate::utils::aliases::{ARuniqueConfig, StrMap, StrVecMap};
use crate::utils::trad::t;
use crate::utils::{
    constante::session_key::session::CSRF_TOKEN_KEY, crypto::csrf::unmask_csrf_token,
};

use axum::{
    body::Body,
    http::{Method, Request, StatusCode},
    response::{IntoResponse, Response},
};
use subtle::ConstantTimeEq;

/// Parsed and CSRF-validated form data extracted from the request.
/// On GET: contains query params, csrf_valid = true.
/// On POST: contains body params, csrf_valid = CSRF check result.
#[derive(Clone)]
pub struct Prisme {
    /// Parsed body/query data. **Crate-private**: user code can NOT read the raw
    /// body without going through the CSRF gate (see anomaly C2). External access
    /// only via `checked_data()` (fail-closed) or `req.form()`.
    pub(crate) data: StrMap,
    pub csrf_valid: bool,
}

impl Prisme {
    /// **Fail-closed** accessor: returns the body data only if CSRF is valid.
    /// The only door into the body for a user handler that doesn't go through
    /// `req.form()`. On invalid CSRF → `None` (a forged request sees no data).
    pub fn checked_data(&self) -> Option<&StrMap> {
        if self.csrf_valid {
            Some(&self.data)
        } else {
            None
        }
    }

    /// **Test-only.** Builds a `Prisme` with arbitrary data.
    ///
    /// The real pipeline goes through [`prisme_pipeline`]; this constructor only
    /// exists for integration tests (a separate crate) that build a `Request` by
    /// hand. Never use this in production code: it bypasses CSRF validation.
    #[doc(hidden)]
    pub fn for_test(data: StrMap, csrf_valid: bool) -> Self {
        Self { data, csrf_valid }
    }
}

/// Non-generic pipeline: Sentinel → Aegis → CSRF check.
/// Runs on every request — aegis handles GET (query params) and POST (body).
pub async fn prisme_pipeline<S>(req: Request<Body>, state: &S) -> Result<Prisme, Response>
where
    S: Send + Sync,
{
    let config = req
        .extensions()
        .get::<ARuniqueConfig>()
        .cloned()
        .ok_or_else(|| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                t("forms.config_not_found").to_string(),
            )
                .into_response()
        })?;

    sentinel(&req, &config).map_err(|boxed| *boxed)?;

    let csrf_session = req
        .extensions()
        .get::<crate::utils::csrf::CsrfToken>()
        .cloned()
        .ok_or_else(|| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                t("csrf.missing").to_string(),
            )
                .into_response()
        })?;

    let method = req.method().clone();

    let content_type = req
        .headers()
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .to_string();

    // Read before `aegis` consumes the request body. Header names are
    // case-insensitive lookups on `HeaderMap`, so this matches `X-CSRF-Token`
    // regardless of the casing a client sends.
    let header_token = req
        .headers()
        .get("x-csrf-token")
        .and_then(|v| v.to_str().ok())
        .map(str::to_string);

    let parsed = aegis(req, state, config, &content_type).await?;

    let csrf_valid = check_csrf(
        &parsed,
        csrf_session.as_str(),
        &method,
        header_token.as_deref(),
    );
    let data = convert_for_form(parsed);

    Ok(Prisme { data, csrf_valid })
}

/// **Single** source of truth for the per-method CSRF policy: only GET/HEAD (safe,
/// no expected side effect) are exempt. **Every** other method — POST/PUT/PATCH/DELETE,
/// but also OPTIONS/TRACE/unknown methods — requires a valid token (fail-closed).
/// Shared by the pipeline (`check_csrf`) and the guard in `Request::form()` so they
/// can never drift apart.
pub(crate) fn csrf_required(method: &Method) -> bool {
    !matches!(*method, Method::GET | Method::HEAD)
}

/// **Single** source of truth for the per-path CSRF exemption policy: `true` if
/// `path` is in `exempt_paths` (webhooks with their own signature check). An
/// exempt path only skips **validation** — the CSRF token is still generated and
/// injected unconditionally by `csrf_middleware`, so `Request`/`RuniqueContext`
/// keep working normally on exempt routes for anything unrelated to CSRF
/// (session, template context, etc.); see the CSRF docs for the full behavior.
pub(crate) fn is_csrf_exempt(path: &str, exempt_paths: &[String]) -> bool {
    exempt_paths.iter().any(|p| p == path)
}

/// Returns true if CSRF is valid or not required (safe method).
///
/// The token is read from the `csrf_token` body/query field first, falling back to the
/// `X-CSRF-Token` header when absent — needed for JSON/fetch clients that can't add a
/// hidden form field. Safe: a plain cross-site `<form>` can't set custom headers at all,
/// and a `fetch`/XHR that does triggers a CORS preflight the browser only lets through
/// for origins the app explicitly allowed (`CorsConfig`, disabled by default, and its
/// `any_origin() + allow_credentials(true)` combination is a build-time error) — so this
/// fallback opens no path a body-only check didn't already have to guard against.
fn check_csrf(
    parsed: &StrVecMap,
    csrf_session: &str,
    method: &Method,
    header_token: Option<&str>,
) -> bool {
    if !csrf_required(method) {
        return true;
    }
    let body_token = parsed
        .get(CSRF_TOKEN_KEY)
        .and_then(|v| v.last())
        .map(String::as_str);
    body_token
        .or(header_token)
        .map(|s| match unmask_csrf_token(s) {
            Ok(unmasked) => bool::from(unmasked.as_bytes().ct_eq(csrf_session.as_bytes())),
            Err(_) => false,
        })
        .unwrap_or(false)
}

fn convert_for_form(parsed: StrVecMap) -> StrMap {
    parsed
        .into_iter()
        .map(|(k, v)| {
            if k == CSRF_TOKEN_KEY {
                (k, v.into_iter().next().unwrap_or_default())
            } else {
                (k, v.join(","))
            }
        })
        .collect()
}

#[cfg(test)]
mod checked_data_tests {
    use super::*;

    /// C2: `checked_data` is fail-closed — None as long as CSRF isn't validated,
    /// even if `.data` (raw) contains fields.
    #[test]
    fn checked_data_gates_on_csrf_valid() {
        let mut data = StrMap::new();
        data.insert("field".to_string(), "value".to_string());

        let invalid = Prisme {
            data: data.clone(),
            csrf_valid: false,
        };
        assert!(invalid.checked_data().is_none(), "CSRF KO → aucune donnée");
        assert!(
            !invalid.data.is_empty(),
            ".data reste peuplé en interne (pub(crate)) — seul l'accès externe est fermé"
        );

        let valid = Prisme {
            data,
            csrf_valid: true,
        };
        assert!(valid.checked_data().is_some(), "CSRF OK → données dispo");
    }

    /// C5: only GET/HEAD are exempt; every other method (including
    /// OPTIONS/TRACE) requires a token (fail-closed). Single source of the policy.
    #[test]
    fn csrf_required_only_exempts_safe_methods() {
        assert!(!csrf_required(&Method::GET), "GET exempté");
        assert!(!csrf_required(&Method::HEAD), "HEAD exempté");
        for m in [
            Method::POST,
            Method::PUT,
            Method::PATCH,
            Method::DELETE,
            Method::OPTIONS,
            Method::TRACE,
        ] {
            assert!(csrf_required(&m), "{m} doit exiger un token CSRF");
        }
    }

    #[test]
    fn is_csrf_exempt_matches_exact_path_only() {
        let exempt = vec!["/webhooks/stripe".to_string(), "/health".to_string()];
        assert!(is_csrf_exempt("/webhooks/stripe", &exempt));
        assert!(is_csrf_exempt("/health", &exempt));
        assert!(
            !is_csrf_exempt("/webhooks/stripe/extra", &exempt),
            "pas de match par préfixe"
        );
        assert!(!is_csrf_exempt("/other", &exempt));
        assert!(!is_csrf_exempt("/webhooks/stripe", &[]), "liste vide");
    }
}
