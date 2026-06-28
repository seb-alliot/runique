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
use tracing::warn;
use uuid::Uuid;

/// Staging dirs older than this are considered orphaned by a rejected upload.
const STAGING_TTL_SECS: u64 = 3600;

pub async fn parse_multipart(
    mut multipart: Multipart,
    upload_dir: &Path,
    max_upload_mb: u64,
    max_text_field_kb: usize,
) -> Result<StrVecMap, Response> {
    let max_file_bytes = max_upload_mb.saturating_mul(1024).saturating_mul(1024);
    let max_text_bytes = max_text_field_kb.saturating_mul(1024);

    let mut data: StrVecMap = HashMap::new();
    // Staging dir (under upload_dir → même filesystem, donc le rename de finalize()
    // est atomique). Les fichiers y restent jusqu'à ce que `FileField::finalize` les
    // committe vers leur destination servie — APRÈS CSRF + validation. Créé à la
    // demande au premier vrai fichier (aucun accès disque pour un form texte seul).
    let mut tmp_dir: Option<PathBuf> = None;

    // Best-effort : purge des staging laissés par des uploads précédemment rejetés.
    sweep_stale_staging(upload_dir).await;

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
                let dir = upload_dir.join(format!(".staging-{}", Uuid::new_v4()));
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

            // Stream into staging — the file handle is scoped to this block
            // to ensure its closure before any staging cleanup.
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
                if let Some(ref tmp) = tmp_dir
                    && let Err(err) = tokio::fs::remove_dir_all(tmp).await
                {
                    warn!(dir = %tmp.display(), error = %err, "staging cleanup after stream error failed");
                }
                return Err(e);
            }

            // Chemin de staging : finalize() le committera en destination servie.
            // Pas de commit eager ici → aucune écriture servie avant CSRF/validation.
            data.entry(name)
                .or_default()
                .push(tmp_path.to_string_lossy().to_string());
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
                    if let Some(ref tmp) = tmp_dir
                        && let Err(err) = tokio::fs::remove_dir_all(tmp).await
                    {
                        warn!(dir = %tmp.display(), error = %err, "staging cleanup after text-field error failed");
                    }
                    return Err(e);
                }
            }
        }
    }

    // Pas de commit ici : les fichiers restent en staging. `FileField::finalize`
    // (le seul committer) les déplacera en destination servie après CSRF + validation.
    // En cas de rejet (CSRF/validation), le staging est purgé par `sweep_stale_staging`
    // au prochain upload (best-effort, TTL).

    Ok(data)
}

/// Best-effort purge des dossiers `.staging-*` orphelins (uploads rejetés avant
/// `finalize`). Supprime ceux plus vieux que `STAGING_TTL_SECS`. Les échecs sont
/// loggés, jamais avalés silencieusement.
async fn sweep_stale_staging(upload_dir: &Path) {
    let mut entries = match tokio::fs::read_dir(upload_dir).await {
        Ok(e) => e,
        Err(_) => return, // upload_dir pas encore créé : rien à purger
    };
    let now = std::time::SystemTime::now();
    loop {
        let entry = match entries.next_entry().await {
            Ok(Some(e)) => e,
            Ok(None) => break,
            Err(err) => {
                warn!(dir = %upload_dir.display(), error = %err, "staging sweep: read_dir failed");
                break;
            }
        };
        let name = entry.file_name();
        let Some(name) = name.to_str() else { continue };
        if !name.starts_with(".staging-") {
            continue;
        }
        let path = entry.path();
        let age = entry
            .metadata()
            .await
            .ok()
            .and_then(|m| m.modified().ok())
            .and_then(|t| now.duration_since(t).ok())
            .map(|d| d.as_secs())
            .unwrap_or(0);
        if age > STAGING_TTL_SECS
            && let Err(err) = tokio::fs::remove_dir_all(&path).await
        {
            warn!(dir = %path.display(), error = %err, "staging sweep: remove failed");
        }
    }
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

#[cfg(test)]
mod staging_tests {
    use super::*;
    use axum::body::Body;
    use axum::extract::FromRequest;
    use axum::http::Request;

    fn multipart_req(boundary: &str, field: &str, filename: &str, content: &str) -> Request<Body> {
        let body = format!(
            "--{b}\r\nContent-Disposition: form-data; name=\"{f}\"; filename=\"{fn}\"\r\nContent-Type: application/octet-stream\r\n\r\n{c}\r\n--{b}--\r\n",
            b = boundary, f = field, fn = filename, c = content
        );
        Request::builder()
            .method("POST")
            .header(
                "content-type",
                format!("multipart/form-data; boundary={boundary}"),
            )
            .body(Body::from(body))
            .unwrap()
    }

    /// L'upload ne doit PAS être commité en racine media_root pendant le parse :
    /// il reste en staging (`.staging-*`), `finalize` committera plus tard, après
    /// CSRF + validation. Sécurise C1 (pas d'écriture servie avant contrôle).
    #[tokio::test]
    async fn parse_multipart_stages_file_without_eager_commit() {
        let media = std::env::temp_dir().join(format!("rq_pm_{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&media).unwrap();

        let req = multipart_req("BNDRY", "avatar", "a.png", "HELLO");
        let mp = Multipart::from_request(req, &()).await.unwrap();

        let parsed = parse_multipart(mp, &media, 10, 64).await.unwrap();

        let path = parsed.get("avatar").expect("champ avatar")[0].clone();
        let p = Path::new(&path);

        assert!(
            path.contains(".staging"),
            "chemin doit pointer vers staging: {path}"
        );
        assert!(p.exists(), "fichier présent en staging");

        // Pas de commit eager : la racine media_root ne contient que le dossier staging,
        // pas le fichier final directement.
        let filename = p.file_name().unwrap();
        assert!(
            !media.join(filename).exists(),
            "aucun fichier commité en racine media_root"
        );

        let _ = std::fs::remove_dir_all(&media);
    }
}
