use crate::forms::field::RuniqueForm;
use crate::utils::trad::t;
use crate::forms::prisme::{aegis, csrf_gate, sentinel};
use crate::utils::aliases::{ARuniqueConfig, ATera, StrMap, StrVecMap};

use axum::{
    body::Body,
    extract::FromRequest,
    http::{Request, StatusCode},
    response::{IntoResponse, Response},
};

/// Prism: Sentinel -> CSRF -> Aegis pipeline (extraction)
pub struct Prisme<T>(pub T);

/// Reusable pipeline function: Sentinel -> CSRF -> Aegis
pub async fn prisme<S, T>(req: Request<Body>, state: &S) -> Result<Prisme<T>, Response>
where
    S: Send + Sync,
    T: RuniqueForm,
{
    // Sentinel: dependencies + access rules
    let tera = req.extensions().get::<ATera>().cloned().ok_or_else(|| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            t("forms.tera_not_configured").to_string(),
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
            (StatusCode::INTERNAL_SERVER_ERROR, t("csrf.missing").to_string()).into_response()
        })?;

    let method = req.method().clone();

    let content_type = req
        .headers()
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .to_string();

    // Aegis (extraction): single read of the body
    let parsed = aegis(req, state, config.clone(), &content_type).await?;

    // CSRF gate: early check on parsed data
    if let Some(prisme) =
        csrf_gate::<T>(&parsed, csrf_session.as_str(), tera.clone(), &method).await?
    {
        return Ok(prisme);
    }

    let form_data = convert_for_form(parsed);

    // ← Changed: build_with_data already creates the renderer, no need to set_tera afterwards
    let form = T::build_with_data(&form_data, tera, csrf_session.as_str(), method).await;

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
    parsed.into_iter().map(|(k, v)| (k, v.join(","))).collect()
}
