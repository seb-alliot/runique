//! Aegis: extraction and normalization of the request body (multipart, urlencoded, JSON, GET).
use crate::middleware::errors::error::too_large;
use crate::utils::{
    aliases::{ARuniqueConfig, StrMap, StrVecMap},
    parse_html::parse_multipart_guarded,
    trad::{t, tf},
};
use axum::{
    body::Body,
    extract::{FromRequest, Multipart},
    http::{Method, Request, StatusCode},
    response::{IntoResponse, Response},
};
use form_urlencoded;
use http_body_util::BodyExt;
use std::collections::HashMap;
use tracing::warn;

/// Largest body read into memory (urlencoded, JSON): the multipart limit is
/// streamed to disk, these would be held in RAM.
const FORM_BODY_LIMIT: usize = 2 * 1024 * 1024;

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
    aegis_guarded(req, state, config, content_type, None)
        .await
        .map(|(data, _staging)| data)
}

/// `aegis`, refusing to write an uploaded file before a valid `csrf_token`
/// field when `upload_gate` holds the session token (see `parse_multipart_guarded`).
pub(crate) async fn aegis_guarded<S>(
    req: Request<Body>,
    state: &S,
    config: ARuniqueConfig,
    content_type: &str,
    upload_gate: Option<&str>,
) -> Result<(StrVecMap, Option<crate::utils::parse_html::Lease>), Response>
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
        return Ok((parsed, None));
    }

    // A body announced over the limit is refused before a byte is read: no
    // upload staged to disk to be deleted. The size a client announces can
    // lie, so the limit still holds while reading (DefaultBodyLimit).
    let body_limit = config
        .static_files
        .max_body_mb
        .saturating_add(1)
        .saturating_mul(1024 * 1024);
    if let Some(len) = req
        .headers()
        .get(axum::http::header::CONTENT_LENGTH)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.parse::<u64>().ok())
        && len > body_limit
    {
        return Err(too_large(t("forms.body_too_large").into_owned()));
    }

    let mut staging = None;
    if content_type.starts_with("multipart/form-data") {
        let multipart = Multipart::from_request(req, state).await.map_err(|_e| {
            (
                StatusCode::BAD_REQUEST,
                t("forms.multipart_error").into_owned(),
            )
                .into_response()
        })?;

        let upload_dir = std::path::Path::new(&config.static_files.media_root);
        let (body, dir) = parse_multipart_guarded(
            multipart,
            upload_dir,
            config.static_files.max_upload_mb,
            config.static_files.max_text_field_kb,
            upload_gate,
        )
        .await?;
        parsed = body;
        staging = dir;
    } else {
        // Held to FORM_BODY_LIMIT, not to `DefaultBodyLimit`: that one is sized
        // for multipart bodies streamed to disk, this one is read into memory.
        let bytes = http_body_util::Limited::new(req.into_body(), FORM_BODY_LIMIT)
            .collect()
            .await
            .map_err(|e| {
                if e.downcast_ref::<http_body_util::LengthLimitError>()
                    .is_some()
                {
                    too_large(t("forms.body_too_large").into_owned())
                } else {
                    (StatusCode::BAD_REQUEST, t("forms.body_error").into_owned()).into_response()
                }
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

    Ok((parsed, staging))
}
