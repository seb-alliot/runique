use crate::formulaire::builder_form::trait_form::RuniqueForm;
use crate::settings::Settings;
use crate::utils::parse_html::parse_multipart;
use axum::{
    body::Body,
    extract::{FromRef, FromRequest, Multipart},
    http::{Request, StatusCode},
    response::{IntoResponse, Response},
};
use http_body_util::BodyExt;
use std::{collections::HashMap, path::Path, sync::Arc};
use tera::Tera;


// ... tes imports restent identiques ...

pub struct ExtractForm<T>(pub T);

impl<S, T> FromRequest<S> for ExtractForm<T>
where
    S: Send + Sync,
    T: RuniqueForm,
    Arc<Tera>: FromRef<S>,
{
    type Rejection = Response;

    async fn from_request(req: Request<Body>, state: &S) -> Result<Self, Self::Rejection> {
        let tera = Arc::<Tera>::from_ref(state);
        let config = Settings::default_values();

        let content_type = req
            .headers()
            .get("content-type")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("")
            .to_string();

        let mut parsed: HashMap<String, Vec<String>> = HashMap::new();

        if content_type.starts_with("multipart/form-data") {
            let multipart = Multipart::from_request(req, state)
                .await
                .map_err(|_| (StatusCode::BAD_REQUEST, "Multipart error").into_response())?;

            let upload_dir = Path::new(&config.media_root);
            parsed = parse_multipart(multipart, upload_dir).await?;

        } else {
            let bytes = req
                .into_body()
                .collect()
                .await
                .map_err(|_| (StatusCode::BAD_REQUEST, "Body error").into_response())?
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

        let mut form = T::build_with_current_data(&form_data_for_validation, tera.clone());

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