use crate::config::RuniqueConfig;
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

/// Aegis : extraction unique du body (multipart/urlencoded/json) et normalisation.
pub async fn aegis<S>(
    req: Request<Body>,
    state: &S,
    config: Arc<RuniqueConfig>,
    content_type: &str,
) -> Result<HashMap<String, Vec<String>>, Response>
where
    S: Send + Sync,
{
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

    Ok(parsed)
}
