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
    /// Données du corps/query parsées. **Privé au crate** : le code utilisateur ne peut
    /// PAS lire le body brut sans passer par la porte CSRF (cf. anomalie C2). Accès
    /// externe uniquement via `checked_data()` (fail-closed) ou `req.form()`.
    pub(crate) data: StrMap,
    pub csrf_valid: bool,
}

impl Prisme {
    /// Accesseur **fail-closed** : renvoie les données du corps uniquement si la CSRF
    /// est valide. Seule porte d'accès au body depuis un handler utilisateur qui ne
    /// passe pas par `req.form()`. Sur CSRF invalide → `None` (la requête forgée ne
    /// voit aucune donnée).
    pub fn checked_data(&self) -> Option<&StrMap> {
        if self.csrf_valid {
            Some(&self.data)
        } else {
            None
        }
    }

    /// **Test-only.** Construit un `Prisme` avec des données arbitraires.
    ///
    /// Le pipeline réel passe par [`prisme_pipeline`] ; ce constructeur n'existe que pour
    /// les tests d'intégration (crate séparée) qui fabriquent une `Request` à la main.
    /// Ne jamais l'utiliser en code de production : il court-circuite la validation CSRF.
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

    let parsed = aegis(req, state, config, &content_type).await?;

    let csrf_valid = check_csrf(&parsed, csrf_session.as_str(), &method);
    let data = convert_for_form(parsed);

    Ok(Prisme { data, csrf_valid })
}

/// Source **unique** de la politique CSRF par méthode HTTP : seules GET/HEAD (sûres, sans
/// effet de bord attendu) sont exemptées. **Toute** autre méthode — POST/PUT/PATCH/DELETE,
/// mais aussi OPTIONS/TRACE/méthodes inconnues — exige un token valide (fail-closed).
/// Partagée par le pipeline (`check_csrf`) et la garde de `Request::form()` pour qu'elles
/// ne puissent jamais diverger.
pub(crate) fn csrf_required(method: &Method) -> bool {
    !matches!(*method, Method::GET | Method::HEAD)
}

/// Returns true if CSRF is valid or not required (safe method).
fn check_csrf(parsed: &StrVecMap, csrf_session: &str, method: &Method) -> bool {
    if !csrf_required(method) {
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
            ".data reste peuplé en interne (pub(crate)) — seul l'accès externe est fermé"
        );

        let valid = Prisme {
            data,
            csrf_valid: true,
        };
        assert!(valid.checked_data().is_some(), "CSRF OK → données dispo");
    }

    /// C5 : seules GET/HEAD sont exemptées ; toute autre méthode (y compris
    /// OPTIONS/TRACE) exige un token (fail-closed). Source unique de la politique.
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
}
