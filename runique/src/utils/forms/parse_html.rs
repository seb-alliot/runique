use crate::{utils::aliases::StrVecMap, utils::trad::t};
use axum::{
    extract::Multipart,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use futures_util::StreamExt;
use std::{collections::HashMap, path::Path};
use tokio::io::AsyncWriteExt;
use uuid::Uuid;

pub async fn parse_multipart(
    mut multipart: Multipart,
    upload_dir: &Path,
) -> Result<StrVecMap, Response> {
    tokio::fs::create_dir_all(upload_dir).await.map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            t("forms.upload_dir_error").to_string(),
        )
            .into_response()
    })?;

    let mut data: StrVecMap = HashMap::new();

    while let Ok(Some(mut field)) = multipart.next_field().await {
        let name = match field.name() {
            Some(n) => n.to_string(),
            None => continue,
        };

        // --- File case ---
        if let Some(filename) = field.file_name().map(|s| s.to_string()) {
            // No file selected (browser sends filename="" with empty body) — drain and skip
            if filename.is_empty() {
                while field.next().await.is_some() {}
                continue;
            }

            let safe = sanitize_filename(&filename);
            let path = upload_dir.join(&safe);

            let mut file = tokio::fs::File::create(&path).await.map_err(|_| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    t("forms.file_create_error").to_string(),
                )
                    .into_response()
            })?;

            while let Some(chunk) = field.next().await {
                let bytes = chunk.map_err(|_| {
                    (
                        StatusCode::BAD_REQUEST,
                        t("forms.multipart_stream_error").to_string(),
                    )
                        .into_response()
                })?;

                file.write_all(&bytes).await.map_err(|_| {
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        t("forms.file_write_error").to_string(),
                    )
                        .into_response()
                })?;
            }

            data.entry(name)
                .or_default()
                .push(path.to_string_lossy().to_string());
        }
        // --- Text field case ---
        else {
            let text = field.text().await.unwrap_or_default();
            data.entry(name).or_default().push(text);
        }
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
        format!("{}.{}", uuid, ext)
    }
}
