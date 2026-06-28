//! Prisme: non-generic CSRF + body extractor integrated into the Request pipeline.
use crate::forms::prisme::{aegis, sentinel};
use crate::utils::aliases::{ARuniqueConfig, StrMap, StrVecMap};
use crate::utils::trad::t;
use crate::utils::{
    constante::session_key::session::CSRF_TOKEN_KEY, middleware::csrf::unmask_csrf_token,
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
    pub data: StrMap,
    pub csrf_valid: bool,
}

impl Prisme {
    /// Accesseur **fail-closed** : renvoie les données du corps uniquement si la CSRF
    /// est valide. À utiliser dans un handler qui lit des champs du body **sans** passer
    /// par `req.form()` — `.data` brut n'est PAS protégé par la CSRF (cf. anomalie C2).
    pub fn checked_data(&self) -> Option<&StrMap> {
        if self.csrf_valid {
            Some(&self.data)
        } else {
            None
        }
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

    let parsed = aegis(req, state, config, &content_type).await?;

    let csrf_valid = check_csrf(&parsed, csrf_session.as_str(), &method);
    let data = convert_for_form(parsed);

    Ok(Prisme { data, csrf_valid })
}

/// Returns true if CSRF is valid or not required (GET/HEAD).
fn check_csrf(parsed: &StrVecMap, csrf_session: &str, method: &Method) -> bool {
    if method == Method::GET || method == Method::HEAD {
        return true;
    }
    parsed
        .get(CSRF_TOKEN_KEY)
        .and_then(|v| v.last())
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

    /// C2 : `checked_data` est fail-closed — None tant que la CSRF n'est pas validée,
    /// même si `.data` (brut) contient des champs.
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
            ".data brut reste accessible (legacy)"
        );

        let valid = Prisme {
            data,
            csrf_valid: true,
        };
        assert!(valid.checked_data().is_some(), "CSRF OK → données dispo");
    }
}
