use crate::{
    utils::aliases::StrVecMap,
    utils::trad::{t, tf},
};
use axum::{
    extract::Multipart,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use futures_util::StreamExt;
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
    tokio::fs::create_dir_all(upload_dir).await.map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            t("forms.upload_dir_error").to_string(),
        )
            .into_response()
    })?;

    let max_file_bytes = max_upload_mb * 1024 * 1024;
    let max_text_bytes = max_text_field_kb * 1024;

    // Répertoire temporaire pour l'upload atomique :
    // tous les fichiers sont d'abord écrits ici, puis déplacés en une fois
    // vers leur destination finale. En cas d'erreur, le tmp est supprimé
    // entièrement — aucun fichier orphelin ne subsiste dans upload_dir.
    let tmp_dir = upload_dir.join(format!("tmp-{}", Uuid::new_v4()));
    tokio::fs::create_dir_all(&tmp_dir).await.map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            t("forms.upload_dir_error").to_string(),
        )
            .into_response()
    })?;

    let mut data: StrVecMap = HashMap::new();
    // (chemin tmp, chemin final, nom du champ)
    let mut pending_files: Vec<(PathBuf, PathBuf, String)> = Vec::new();

    while let Ok(Some(mut field)) = multipart.next_field().await {
        let name = match field.name() {
            Some(n) => n.to_string(),
            None => continue,
        };

        // --- Champ fichier ---
        if let Some(filename) = field.file_name().map(std::string::ToString::to_string) {
            // Aucun fichier sélectionné (filename="" + body vide) — ignorer
            if filename.is_empty() {
                while field.next().await.is_some() {}
                continue;
            }

            let safe = sanitize_filename(&filename);
            let tmp_path = tmp_dir.join(&safe);
            let final_path = upload_dir.join(&safe);

            // Stream dans le tmp — le handle de fichier est scoped à ce bloc
            // pour garantir sa fermeture avant tout cleanup du tmp_dir.
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
                    written += bytes.len() as u64;
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
                let _ = tokio::fs::remove_dir_all(&tmp_dir).await;
                return Err(e);
            }

            pending_files.push((tmp_path, final_path, name));
        }
        // --- Champ texte ---
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
                    if bytes.len() + b.len() > max_text_bytes {
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
                    let _ = tokio::fs::remove_dir_all(&tmp_dir).await;
                    return Err(e);
                }
            }
        }
    }

    // Commit : déplacer tous les fichiers du tmp vers leur destination finale.
    // tmp_dir est dans upload_dir → même filesystem → rename atomique garanti.
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

    // Supprime le répertoire tmp maintenant vide
    let _ = tokio::fs::remove_dir(&tmp_dir).await;

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
