//! CSRF gate of the Prisme pipeline: validates the token on mutating requests (POST/PUT/PATCH/DELETE).
use crate::forms::{extractor::Prisme, field::RuniqueForm};
use crate::utils::{
    aliases::{StrMap, StrVecMap},
    constante::session_key::session::CSRF_TOKEN_KEY,
    middleware::csrf::unmask_csrf_token,
    trad::t,
};

use axum::{http::Method, response::Response};
use std::{collections::HashMap, sync::Arc};
use subtle::ConstantTimeEq;
use tera::Tera;

/// CSRF gate: verification of the token in the parsed data.
/// On GET/HEAD without submitted token (initial load), validation is ignored.
/// Returns Some(Prisme) if invalid/missing (empty form with error), None otherwise.
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

    // GET/HEAD: no CSRF validation (token not consumed + middleware strips the URL)
    if method == Method::GET || method == Method::HEAD {
        return Ok(None);
    }

    // Unmasks the submitted token (base64 → hex) then compares in constant-time
    // to the raw session token to avoid a timing attack.
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
