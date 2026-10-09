//! Aegis: extraction and normalization of the request body (multipart, urlencoded, JSON, GET).
use crate::utils::{
    aliases::{ARuniqueConfig, StrMap, StrVecMap},
    parse_html::parse_multipart_guarded,
    trad::{t, tf},
};
use axum::{
    RequestExt,
    body::Body,
    extract::{FromRequest, Multipart},
    http::{Method, Request, StatusCode},
    response::{IntoResponse, Response},
};
use form_urlencoded;
use http_body_util::BodyExt;
use std::collections::HashMap;
use tracing::warn;

/// Aegis: unique extraction of the body (multipart/urlencoded/json) and normalization.
/// On GET/HEAD, data is read from query params (including CSRF token).
pub async fn aegis<S>(
    req: Request<Body>,
    state: &S,
    config: ARuniqueConfig,
    content_type: &str,
) -> Result<StrVecMap, Response>
where
    S: Send + Sync,
{
    aegis_guarded(req, state, config, content_type, None).await
}

/// `aegis`, refusing to write an uploaded file before a valid `csrf_token`
/// field when `upload_gate` holds the session token (see `parse_multipart_guarded`).
pub(crate) async fn aegis_guarded<S>(
    req: Request<Body>,
    state: &S,
    config: ARuniqueConfig,
    content_type: &str,
    upload_gate: Option<&str>,
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
        parsed = parse_multipart_guarded(
            multipart,
            upload_dir,
            config.static_files.max_upload_mb,
            config.static_files.max_text_field_kb,
            upload_gate,
        )
        .await?;
    } else {
        // Bounded like the multipart body (`DefaultBodyLimit`, set at build):
        // a raw `collect()` would read a body of any size into memory.
        let bytes = req
            .with_limited_body()
            .into_body()
            .collect()
            .await
            .map_err(|_e| {
                (StatusCode::BAD_REQUEST, t("forms.body_error").into_owned()).into_response()
            })?
            .to_bytes();

        if content_type.starts_with("application/x-www-form-urlencoded") {
            for (k, v) in form_urlencoded::parse(&bytes) {
                parsed
                    .entry(k.into_owned())
                    .or_default()
                    .push(v.into_owned());
            }
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
