use crate::config::RuniqueConfig;
use crate::utils::aliases::{StrMap, StrVecMap};
use crate::utils::parse_html::parse_multipart;
use crate::utils::trad::{t, tf};
use axum::{
    body::Body,
    extract::{FromRequest, Multipart},
    http::{Method, Request, StatusCode},
    response::{IntoResponse, Response},
};
use http_body_util::BodyExt;
use std::collections::HashMap;
use std::sync::Arc;
use tracing::warn;

/// Aegis : extraction unique du body (multipart/urlencoded/json) et normalisation.
/// Sur GET/HEAD, les données sont lues depuis les query params (token CSRF inclus).
pub async fn aegis<S>(
    req: Request<Body>,
    state: &S,
    config: Arc<RuniqueConfig>,
    content_type: &str,
) -> Result<StrVecMap, Response>
where
    S: Send + Sync,
{
    let mut parsed: StrVecMap = HashMap::new();

    if req.method() == Method::GET || req.method() == Method::HEAD {
        let query = req.uri().query().unwrap_or("");
        parsed = serde_urlencoded::from_str::<StrMap>(query)
            .unwrap_or_else(|e| {
                warn!("{}", tf("forms.aegis_query_error", &[&e]));
                StrMap::default()
            })
            .into_iter()
            .map(|(k, v)| (k, vec![v]))
            .collect();
        return Ok(parsed);
    }

    if content_type.starts_with("multipart/form-data") {
        let multipart = Multipart::from_request(req, state).await.map_err(|_e| {
            (
                StatusCode::BAD_REQUEST,
                t("forms.multipart_error").into_owned(),
            )
                .into_response()
        })?;

        let upload_dir = std::path::Path::new(&config.static_files.media_root);
        parsed = parse_multipart(multipart, upload_dir).await?;
    } else {
        let bytes = req
            .into_body()
            .collect()
            .await
            .map_err(|_e| {
                (StatusCode::BAD_REQUEST, t("forms.body_error").into_owned()).into_response()
            })?
            .to_bytes();

        if content_type.starts_with("application/x-www-form-urlencoded") {
            parsed = serde_urlencoded::from_bytes::<StrMap>(&bytes)
                .unwrap_or_else(|e| {
                    warn!("{}", tf("forms.aegis_urlencoded_error", &[&e]));
                    StrMap::default()
                })
                .into_iter()
                .map(|(k, v)| (k, vec![v]))
                .collect();
        } else if content_type.starts_with("application/json") {
            parsed = serde_json::from_slice::<StrMap>(&bytes)
                .unwrap_or_else(|e| {
                    warn!("{}", tf("forms.aegis_json_error", &[&e]));
                    StrMap::default()
                })
                .into_iter()
                .map(|(k, v)| (k, vec![v]))
                .collect();
        }
    }

    Ok(parsed)
}
