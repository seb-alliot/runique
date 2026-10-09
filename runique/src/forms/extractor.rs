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

    /// **Test-only** (`test-utils` feature). Builds a `Prisme` with arbitrary data.
    ///
    /// The real pipeline goes through [`prisme_pipeline`]; this constructor only
    /// exists for integration tests (a separate crate) that build a `Request` by
    /// hand. It bypasses CSRF validation, hence out of normal builds.
    #[cfg(feature = "test-utils")]
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

    sentinel(&req).await.map_err(|boxed| *boxed)?;

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

    // A multipart body writes its files to disk while it's read: without a
    // valid header token, the `csrf_token` field must come before the first
    // file, or nothing is written (an exempt path keeps its own check).
    let exempt = req
        .extensions()
        .get::<crate::utils::aliases::AEngine>()
        .is_some_and(|engine| is_csrf_exempt(req.uri().path(), &engine.csrf_exempt_paths));
    let header_valid = header_token
        .as_deref()
        .is_some_and(|token| token_matches(token, csrf_session.as_str()));
    let upload_gate =
        (csrf_required(&method) && !exempt && !header_valid).then(|| csrf_session.as_str());

    let parsed = aegis::aegis_guarded(req, state, config, &content_type, upload_gate).await?;

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
        .is_some_and(|token| token_matches(token, csrf_session))
}

/// Whether a masked token from the request is the session's, compared in
/// constant time (`ct_eq`) so a wrong guess can't be told from a right one by
/// response time.
pub(crate) fn token_matches(masked: &str, csrf_session: &str) -> bool {
    unmask_csrf_token(masked)
        .is_ok_and(|unmasked| bool::from(unmasked.as_bytes().ct_eq(csrf_session.as_bytes())))
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

/// Written from cargo-mutants survivors (2026-10-02): the CSRF check that
/// protects every HTML form going through Prisme had no POST test at all —
/// it could have accepted anything without a test failing.
#[cfg(test)]
mod csrf_tests {
    use super::*;
    use crate::utils::crypto::csrf::mask_csrf_token;

    const SESSION: &str = "a3f1c2d4e5b60718293a4b5c6d7e8f90a3f1c2d4e5b60718293a4b5c6d7e8f90";
    const OTHER: &str = "0000000000000000000000000000000000000000000000000000000000000000";

    fn body(token: Option<&str>) -> StrVecMap {
        let mut parsed = StrVecMap::new();
        if let Some(t) = token {
            parsed.insert(CSRF_TOKEN_KEY.to_string(), vec![t.to_string()]);
        }
        parsed
    }

    #[test]
    fn a_post_needs_the_session_token() {
        let good = mask_csrf_token(SESSION).unwrap();
        let other = mask_csrf_token(OTHER).unwrap();
        assert!(check_csrf(&body(Some(&good)), SESSION, &Method::POST, None));
        assert!(
            !check_csrf(&body(Some(&other)), SESSION, &Method::POST, None),
            "another token"
        );
        assert!(
            !check_csrf(&body(Some("garbage")), SESSION, &Method::POST, None),
            "not a token"
        );
        assert!(
            !check_csrf(&body(None), SESSION, &Method::POST, None),
            "no token"
        );
    }

    #[test]
    fn the_header_is_a_fallback_never_an_override() {
        let good = mask_csrf_token(SESSION).unwrap();
        let other = mask_csrf_token(OTHER).unwrap();
        assert!(
            check_csrf(&body(None), SESSION, &Method::POST, Some(&good)),
            "header alone"
        );
        assert!(
            !check_csrf(&body(Some(&other)), SESSION, &Method::POST, Some(&good)),
            "a wrong body token isn't rescued by a right header"
        );
    }

    #[test]
    fn every_unsafe_method_is_checked_safe_ones_are_not() {
        for method in [Method::PUT, Method::PATCH, Method::DELETE] {
            assert!(!check_csrf(&body(None), SESSION, &method, None), "{method}");
        }
        for method in [Method::GET, Method::HEAD] {
            assert!(check_csrf(&body(None), SESSION, &method, None), "{method}");
        }
    }

    #[test]
    fn the_csrf_token_is_never_joined_other_fields_are() {
        let mut parsed = StrVecMap::new();
        parsed.insert(
            CSRF_TOKEN_KEY.to_string(),
            vec!["first".into(), "second".into()],
        );
        parsed.insert("tags".to_string(), vec!["a".into(), "b".into()]);
        let data = convert_for_form(parsed);
        assert_eq!(data[CSRF_TOKEN_KEY], "first");
        assert_eq!(data["tags"], "a,b");
    }
}
