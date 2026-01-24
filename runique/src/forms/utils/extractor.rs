use crate::config::RuniqueConfig;
use crate::forms::field::RuniqueForm;
use crate::utils::parse_html::parse_multipart;
use axum::{
    body::Body,
    extract::{FromRequest, Multipart},
    http::{Request, StatusCode},
    response::{IntoResponse, Response},
};
use http_body_util::BodyExt;
use std::collections::HashMap;
use std::sync::Arc;
use tera::Tera;

pub struct ExtractForm<T>(pub T);

impl<S, T> FromRequest<S> for ExtractForm<T>
where
    S: Send + Sync,
    T: RuniqueForm,
{
    type Rejection = Response;

    async fn from_request(req: Request<Body>, state: &S) -> Result<Self, Self::Rejection> {
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

        let content_type = req
            .headers()
            .get("content-type")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("")
            .to_string();

        // Récupérer le token CSRF de session
        let csrf_token = req
            .extensions()
            .get::<crate::utils::csrf::CsrfToken>()
            .cloned()
            .ok_or_else(|| {
                (StatusCode::INTERNAL_SERVER_ERROR, "CSRF token not found").into_response()
            })?;

        let mut parsed: HashMap<String, Vec<String>> = HashMap::new();

        if content_type.starts_with("multipart/form-data") {
            let multipart = Multipart::from_request(req, state)
                .await
                .map_err(|_e| (StatusCode::BAD_REQUEST, "Multipart error").into_response())?;

            let upload_dir = std::path::Path::new(&config.static_files.media_root);
            parsed = parse_multipart(multipart, upload_dir).await?;
        } else {
            let bytes = req
                .into_body()
                .collect()
                .await
                .map_err(|_e| (StatusCode::BAD_REQUEST, "Body error").into_response())?
                .to_bytes();

            if content_type.starts_with("application/x-www-form-urlencoded") {
                parsed = serde_urlencoded::from_bytes::<HashMap<String, String>>(&bytes)
                    .unwrap_or_default()
                    .into_iter()
                    .map(|(k, v)| (k, vec![v]))
                    .collect();
            } else if content_type.starts_with("application/json") {
                parsed = serde_json::from_slice::<HashMap<String, String>>(&bytes)
                    .unwrap_or_default()
                    .into_iter()
                    .map(|(k, v)| (k, vec![v]))
                    .collect();
            }
        }

        let form_data_for_validation = convert_for_form(parsed);

        // Passer le token CSRF au build
        let mut form = T::build_with_data(
            &form_data_for_validation,
            tera.clone(),
            csrf_token.as_str(), // ← Token de session (non masqué)
        )
        .await;

        form.get_form_mut().set_tera(tera);

        Ok(ExtractForm(form))
    }
}

fn convert_for_form(parsed: HashMap<String, Vec<String>>) -> HashMap<String, String> {
    parsed
        .into_iter()
        .filter_map(|(k, mut v)| v.pop().map(|val| (k, val)))
        .collect()
}
