use crate::config::RuniqueConfig;
use crate::forms::field::RuniqueForm;
use crate::forms::utils::prisme::{aegis, csrf_gate, sentinel};
use axum::{
    body::Body,
    extract::FromRequest,
    http::{Request, StatusCode},
    response::{IntoResponse, Response},
};
use std::collections::HashMap;
use std::sync::Arc;
use tera::Tera;

/// Prisme : pipeline Sentinel -> CSRF -> Aegis (extraction)
pub struct Prisme<T>(pub T);
/// Alias rétro-compatibilité
pub type ExtractForm<T> = Prisme<T>;

/// Fonction pipeline réutilisable : Sentinel -> CSRF -> Aegis
pub async fn prisme<S, T>(req: Request<Body>, state: &S) -> Result<Prisme<T>, Response>
where
    S: Send + Sync,
    T: RuniqueForm,
{
    // Sentinel : dépendances + règles d'accès
    let tera = req
        .extensions()
        .get::<Arc<Tera>>()
        .cloned()
        .ok_or_else(|| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Tera not found in extensions",
            )
                .into_response()
        })?;

    let config = req
        .extensions()
        .get::<Arc<RuniqueConfig>>()
        .cloned()
        .ok_or_else(|| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Config not found in extensions",
            )
                .into_response()
        })?;

    sentinel(&req, &config)?;

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

    let form_data_for_validation = convert_for_form(parsed);

    let mut form = T::build_with_data(
        &form_data_for_validation,
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

fn convert_for_form(parsed: HashMap<String, Vec<String>>) -> HashMap<String, String> {
    parsed
        .into_iter()
        .filter_map(|(k, mut v)| v.pop().map(|val| (k, val)))
        .collect()
}
