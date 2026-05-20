//! Multipart request parsing — text field extraction and file uploads to disk.
use crate::{
    errors::error::ErrorContext,
    utils::aliases::StrVecMap,
    utils::trad::{t, tf},
};
use axum::{
    extract::Multipart,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use futures_util::StreamExt;
use std::sync::Arc;
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};
use tokio::io::AsyncWriteExt;
use uuid::Uuid;

pub async fn parse_multipart(
    mut multipart: Multipart,
    upload_dir: &Path,
    max_upload_mb: u64,
    max_text_field_kb: usize,
) -> Result<StrVecMap, Response> {
    let max_file_bytes = max_upload_mb.saturating_mul(1024).saturating_mul(1024);
    let max_text_bytes = max_text_field_kb.saturating_mul(1024);

    let mut data: StrVecMap = HashMap::new();
    // (tmp path, final path, field name)
    let mut pending_files: Vec<(PathBuf, PathBuf, String)> = Vec::new();
    // Created lazily on first real file upload — never touches the filesystem for text-only forms.
    let mut tmp_dir: Option<PathBuf> = None;

    while let Ok(Some(mut field)) = multipart.next_field().await {
        let name = match field.name() {
            Some(n) => n.to_string(),
            None => continue,
        };

        // --- File field ---
        if let Some(filename) = field.file_name().map(std::string::ToString::to_string) {
            // No file selected (filename="" + empty body) — ignore
            if filename.is_empty() {
                while field.next().await.is_some() {}
                continue;
            }

            // Lazy init: create upload_dir and tmp dir only on first real file.
            if tmp_dir.is_none() {
                tokio::fs::create_dir_all(upload_dir).await.map_err(|e| {
                    let msg = format!(
                        "{} — path: {:?}, os error: {}",
                        t("forms.upload_dir_error"),
                        upload_dir,
                        e
                    );
                    let ctx = ErrorContext::generic(StatusCode::INTERNAL_SERVER_ERROR, &msg);
                    let mut res = StatusCode::INTERNAL_SERVER_ERROR.into_response();
                    res.extensions_mut().insert(Arc::new(ctx));
                    res
                })?;
                let dir = upload_dir.join(format!("tmp-{}", Uuid::new_v4()));
                tokio::fs::create_dir_all(&dir).await.map_err(|_| {
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        t("forms.upload_dir_error").to_string(),
                    )
                        .into_response()
                })?;
                tmp_dir = Some(dir);
            }
            let cur_tmp = tmp_dir.as_ref().unwrap();

            let safe = sanitize_filename(&filename);
            let tmp_path = cur_tmp.join(&safe);
            let final_path = upload_dir.join(&safe);

            // Stream into tmp — the file handle is scoped to this block
            // to ensure its closure before any tmp_dir cleanup.
            let stream_result: Result<(), Response> = async {
                let mut file = tokio::fs::File::create(&tmp_path).await.map_err(|_| {
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        t("forms.file_create_error").to_string(),
                    )
                        .into_response()
                })?;

                let mut written: u64 = 0;
                while let Some(chunk) = field.next().await {
                    let bytes = chunk.map_err(|_| {
                        (
                            StatusCode::BAD_REQUEST,
                            t("forms.multipart_stream_error").to_string(),
                        )
                            .into_response()
                    })?;
                    written = written.saturating_add(bytes.len() as u64);
                    if written > max_file_bytes {
                        return Err((
                            StatusCode::PAYLOAD_TOO_LARGE,
                            tf("forms.upload_too_large", &[&max_upload_mb]).clone(),
                        )
                            .into_response());
                    }
                    file.write_all(&bytes).await.map_err(|_| {
                        (
                            StatusCode::INTERNAL_SERVER_ERROR,
                            t("forms.file_write_error").to_string(),
                        )
                            .into_response()
                    })?;
                }
                Ok(())
            }
            .await;

            if let Err(e) = stream_result {
                if let Some(ref tmp) = tmp_dir {
                    let _ = tokio::fs::remove_dir_all(tmp).await;
                }
                return Err(e);
            }

            pending_files.push((tmp_path, final_path, name));
        }
        // --- Text field ---
        else {
            let text_result: Result<String, Response> = async {
                let mut bytes: Vec<u8> = Vec::new();
                while let Some(chunk) = field.next().await {
                    let b = chunk.map_err(|_| {
                        (
                            StatusCode::BAD_REQUEST,
                            t("forms.multipart_stream_error").to_string(),
                        )
                            .into_response()
                    })?;
                    if bytes.len().saturating_add(b.len()) > max_text_bytes {
                        return Err((
                            StatusCode::PAYLOAD_TOO_LARGE,
                            t("forms.text_field_too_large").to_string(),
                        )
                            .into_response());
                    }
                    bytes.extend_from_slice(&b);
                }
                Ok(String::from_utf8_lossy(&bytes).into_owned())
            }
            .await;

            match text_result {
                Ok(text) => data.entry(name).or_default().push(text),
                Err(e) => {
                    if let Some(ref tmp) = tmp_dir {
                        let _ = tokio::fs::remove_dir_all(tmp).await;
                    }
                    return Err(e);
                }
            }
        }
    }

    // Commit: move all files from tmp to their final destination.
    // tmp_dir is within upload_dir → same filesystem → atomic rename guaranteed.
    for (tmp_path, final_path, field_name) in pending_files {
        if tokio::fs::rename(&tmp_path, &final_path).await.is_err() {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                t("forms.file_write_error").to_string(),
            )
                .into_response());
        }
        data.entry(field_name)
            .or_default()
            .push(final_path.to_string_lossy().to_string());
    }

    // Remove the now-empty tmp directory
    if let Some(ref tmp) = tmp_dir {
        let _ = tokio::fs::remove_dir(tmp).await;
    }

    Ok(data)
}

fn sanitize_filename(filename: &str) -> String {
    let ext = Path::new(filename)
        .extension()
        .and_then(|e: &std::ffi::OsStr| e.to_str())
        .unwrap_or("");
    let uuid = Uuid::new_v4().to_string();
    if ext.is_empty() {
        uuid
    } else {
        format!("{uuid}.{ext}")
    }
}
