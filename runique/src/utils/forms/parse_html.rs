use crate::aliases::StrVecMap;
use axum::extract::Multipart;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use futures_util::StreamExt;
use std::collections::HashMap;
use std::path::Path;
use tokio::io::AsyncWriteExt;
use uuid::Uuid;

pub async fn parse_multipart(
    mut multipart: Multipart,
    upload_dir: &Path,
) -> Result<StrVecMap, Response> {
    tokio::fs::create_dir_all(upload_dir)
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Upload dir error").into_response())?;

    let mut data: StrVecMap = HashMap::new();

    while let Ok(Some(mut field)) = multipart.next_field().await {
        let name = match field.name() {
            Some(n) => n.to_string(),
            None => continue,
        };

        // --- Cas fichier ---
        if let Some(filename) = field.file_name().map(|s| s.to_string()) {
            let safe = sanitize_filename(&filename);
            let path = upload_dir.join(&safe);

            let mut file = tokio::fs::File::create(&path).await.map_err(|_| {
                (StatusCode::INTERNAL_SERVER_ERROR, "File create error").into_response()
            })?;

            while let Some(chunk) = field.next().await {
                let bytes = chunk.map_err(|_| {
                    (StatusCode::BAD_REQUEST, "Multipart stream error").into_response()
                })?;

                file.write_all(&bytes).await.map_err(|_| {
                    (StatusCode::INTERNAL_SERVER_ERROR, "File write error").into_response()
                })?;
            }

            data.entry(name).or_default().push(safe);
        }
        // --- Cas champ texte ---
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
