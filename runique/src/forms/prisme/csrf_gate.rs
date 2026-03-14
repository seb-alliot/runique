use crate::forms::extractor::Prisme;
use crate::forms::field::RuniqueForm;
use crate::utils::aliases::{StrMap, StrVecMap};
use crate::utils::constante::CSRF_TOKEN_KEY;
use crate::utils::middleware::csrf::unmask_csrf_token;
use crate::utils::trad::t;
use axum::http::Method;
use axum::response::Response;
use std::collections::HashMap;
use std::sync::Arc;
use subtle::ConstantTimeEq;
use tera::Tera;

/// CSRF gate : vérification du token dans les données parsées.
/// Sur GET/HEAD sans token soumis (chargement initial), la validation est ignorée.
/// Retourne Some(Prisme) si invalid/missing (formulaire vide avec erreur), None sinon.
pub async fn csrf_gate<T: RuniqueForm>(
    parsed: &StrVecMap,
    csrf_session: &str,
    tera: Arc<Tera>,
    method: &Method,
) -> Result<Option<Prisme<T>>, Response> {
    let csrf_submitted = parsed
        .get(CSRF_TOKEN_KEY)
        .and_then(|v| v.last())
        .map(|s| s.as_str());

    // GET/HEAD : pas de validation CSRF (token non consommé + middleware strip l'URL)
    if method == Method::GET || method == Method::HEAD {
        return Ok(None);
    }

    // Démasque le token soumis (base64 → hex) puis compare en constant-time
    // au token brut de session pour éviter une attaque par timing.
    let token_valid = csrf_submitted
        .map(|s| match unmask_csrf_token(s) {
            Ok(unmasked) => bool::from(unmasked.as_bytes().ct_eq(csrf_session.as_bytes())),
            Err(_) => false,
        })
        .unwrap_or(false);

    if !token_valid {
        let empty: StrMap = HashMap::new();
        let mut form = T::build_with_data(&empty, tera, csrf_session, method.clone()).await;

        if let Some(csrf_field) = form.get_form_mut().fields.get_mut(CSRF_TOKEN_KEY) {
            csrf_field.set_error(t("csrf.invalid_or_missing").into_owned());
        }
        return Ok(Some(Prisme(form)));
    }

    Ok(None)
}
