use crate::forms::field::RuniqueForm;
use crate::forms::prisme::{aegis, csrf_gate, sentinel};
use crate::formulaire::{auto_sanitize, is_sensitive_field};
use crate::utils::aliases::{ARuniqueConfig, ATera, StrMap, StrVecMap};

use axum::{
    body::Body,
    extract::FromRequest,
    http::{Request, StatusCode},
    response::{IntoResponse, Response},
};

/// Prisme : pipeline Sentinel -> CSRF -> Aegis (extraction)
pub struct Prisme<T>(pub T);

/// Fonction pipeline réutilisable : Sentinel -> CSRF -> Aegis
pub async fn prisme<S, T>(req: Request<Body>, state: &S) -> Result<Prisme<T>, Response>
where
    S: Send + Sync,
    T: RuniqueForm,
{
    // Sentinel : dépendances + règles d'accès
    let tera = req.extensions().get::<ATera>().cloned().ok_or_else(|| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Tera not found in extensions",
        )
            .into_response()
    })?;

    let config = req
        .extensions()
        .get::<ARuniqueConfig>()
        .cloned()
        .ok_or_else(|| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Config not found in extensions",
            )
                .into_response()
        })?;

    sentinel(&req, &config).map_err(|boxed| *boxed)?;

    let csrf_session = req
        .extensions()
        .get::<crate::utils::csrf::CsrfToken>()
        .cloned()
        .ok_or_else(|| {
            (StatusCode::INTERNAL_SERVER_ERROR, "CSRF token not found").into_response()
        })?;

    let content_type = req
        .headers()
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .to_string();

    // Aegis (extraction) : lecture unique du body
    let parsed = aegis(req, state, config.clone(), &content_type).await?;

    // CSRF gate : vérification précoce sur données parsées
    if let Some(prisme) = csrf_gate::<T>(&parsed, csrf_session.as_str(), tera.clone()).await? {
        return Ok(prisme);
    }
    // On récupère le flag depuis la config injectée par le builder
    let mut form_data = convert_for_form(parsed);

    if config.security.sanitize_inputs {
        for (key, value) in form_data.iter_mut() {
            // On ne sanitize pas les champs sensibles (mots de passe)
            if !is_sensitive_field(key) {
                *value = auto_sanitize(value.as_str());
            }
        }
    }

    let mut form = T::build_with_data(
        &form_data, // Maintenant potentiellement nettoyé
        tera.clone(),
        csrf_session.as_str(),
    )
    .await;

    form.get_form_mut().set_tera(tera);

    Ok(Prisme(form))
}

impl<S, T> FromRequest<S> for Prisme<T>
where
    S: Send + Sync,
    T: RuniqueForm,
{
    type Rejection = Response;

    async fn from_request(req: Request<Body>, state: &S) -> Result<Self, Self::Rejection> {
        prisme(req, state).await
    }
}

fn convert_for_form(parsed: StrVecMap) -> StrMap {
    parsed
        .into_iter()
        .filter_map(|(k, mut v)| v.pop().map(|val| (k, val)))
        .collect()
}
