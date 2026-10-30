//! Multipart request parsing — text field extraction and file uploads to disk.
use crate::{
    errors::error::ErrorContext,
    utils::aliases::StrVecMap,
    utils::constante::session_key::session::CSRF_TOKEN_KEY,
    utils::trad::{t, tf},
};
use axum::{
    extract::Multipart,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use futures_util::StreamExt;
use std::collections::HashSet;
use std::fs::File;
use std::sync::{Arc, LazyLock, Mutex};
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};
use tokio::io::AsyncWriteExt;
use tracing::warn;
use uuid::Uuid;

/// Directories this process is still uploading into. A sweep skips them, however
/// old they are: a slow body must not lose its folder to the next request.
/// The set dies with the process, so a directory left by a dead process is not
/// in it and the next multipart removes that directory.
static LIVE_STAGING: LazyLock<Mutex<HashSet<PathBuf>>> =
    LazyLock::new(|| Mutex::new(HashSet::new()));

fn with_live<T>(f: impl FnOnce(&mut HashSet<PathBuf>) -> T) -> T {
    let mut guard = LIVE_STAGING
        .lock()
        .unwrap_or_else(|poison| poison.into_inner());
    f(&mut guard)
}

/// Owns one `.staging-*` directory from the moment it is reserved until the
/// request drops it. `delete_on_drop` is set only once the request holds it:
/// a parser that merely returns the paths leaves the directory for that holder.
pub(crate) struct Lease {
    path: PathBuf,
    /// Exclusive lock, held for the life of this lease. Another process's sweep
    /// fails to take it and leaves the directory alone. The kernel drops the
    /// lock when this process dies.
    _lock: Option<File>,
    delete_on_drop: bool,
}

impl Lease {
    /// Reserves `path` before the directory exists, so a sweep cannot see it
    /// unclaimed.
    pub(crate) fn claim(path: PathBuf) -> Self {
        with_live(|live| {
            live.insert(path.clone());
        });
        Self {
            path,
            _lock: None,
            delete_on_drop: false,
        }
    }

    fn path(&self) -> &Path {
        &self.path
    }

    /// Locks the real directory. A symlink is not opened: locking it would lock
    /// the folder it points at.
    pub(crate) fn lock_dir(&mut self) -> std::io::Result<()> {
        if self._lock.is_some() {
            return Ok(());
        }
        let meta = std::fs::symlink_metadata(&self.path)?;
        if !meta.is_dir() || meta.file_type().is_symlink() {
            return Ok(());
        }
        let file = File::open(&self.path)?;
        lock_exclusive(&file)?;
        self._lock = Some(file);
        Ok(())
    }

    /// The request is over, whatever its result: the directory goes with the lease.
    fn discard(mut self) {
        self.delete_on_drop = true;
    }
}

impl Drop for Lease {
    fn drop(&mut self) {
        with_live(|live| {
            live.remove(&self.path);
        });
        if self.delete_on_drop {
            remove_staging_dir(&self.path);
        }
    }
}

/// The `.staging-*` directory one request created. The last clone removes it,
/// and only it: the name has to be the directory `parse_multipart` made, and a
/// symlink is left untouched so the removal can't be pointed at another folder.
/// Accepted files have already been moved out by `finalize`.
pub(crate) struct StagingDir {
    lease: Lease,
}

impl StagingDir {
    pub(crate) fn from_lease(mut lease: Lease) -> Self {
        lease.delete_on_drop = true;
        Self { lease }
    }
}

impl Drop for StagingDir {
    fn drop(&mut self) {
        // The request is finished. `Lease` removes the directory as it drops.
        self.lease.delete_on_drop = true;
    }
}

/// Removes a real `.staging-*` directory. A symlink under that name is left
/// in place so the removal cannot be aimed at the folder it points at.
fn remove_staging_dir(path: &Path) {
    let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
        return;
    };
    if !name.starts_with(".staging-") {
        return;
    }
    // `symlink_metadata` does not follow a link.
    let Ok(meta) = std::fs::symlink_metadata(path) else {
        return;
    };
    if !meta.is_dir() || meta.file_type().is_symlink() {
        return;
    }
    if let Err(e) = std::fs::remove_dir_all(path) {
        warn!(
            dir = %path.display(),
            error = %e,
            "staged upload directory was not removed"
        );
    }
}

fn lock_exclusive(file: &File) -> std::io::Result<()> {
    // Safety: `file` is an open descriptor. `LOCK_EX` blocks until this process
    // is the only holder; the kernel releases it when the descriptor closes.
    let rc = unsafe { libc::flock(as_raw_fd(file), libc::LOCK_EX) };
    if rc == 0 {
        Ok(())
    } else {
        Err(std::io::Error::last_os_error())
    }
}

/// `false` when another live process already holds the directory.
fn try_lock_exclusive(file: &File) -> bool {
    // Safety: same descriptor rules as `lock_exclusive`. `LOCK_NB` returns at once.
    unsafe { libc::flock(as_raw_fd(file), libc::LOCK_EX | libc::LOCK_NB) == 0 }
}

fn as_raw_fd(file: &File) -> libc::c_int {
    use std::os::unix::io::AsRawFd;
    file.as_raw_fd()
}

/// Reads a multipart body: text fields as they are, files written to a
/// `.staging-*` folder under `upload_dir` (their path as the value), each held
/// to `max_upload_mb`. No CSRF check here: the request pipeline uses
/// `parse_multipart_guarded`.
pub async fn parse_multipart(
    multipart: Multipart,
    upload_dir: &Path,
    max_upload_mb: u64,
    max_text_field_kb: usize,
) -> Result<StrVecMap, Response> {
    // The directory is returned and dropped here on purpose: this function has
    // no request to attach it to. The pipeline (`aegis_guarded`) is what holds
    // it until the request ends. A direct caller still relies on the sweep.
    parse_multipart_guarded(
        multipart,
        upload_dir,
        max_upload_mb,
        max_text_field_kb,
        None,
    )
    .await
    .map(|(data, _staging)| data)
}

/// `parse_multipart`, with `csrf_session` the session token when the request
/// carries no valid `X-CSRF-Token` header: a file is then written only once a
/// `csrf_token` field matching it has been read, earlier in the body (the
/// framework renders that field before every other). A file before it ends the
/// request with a 403, nothing written.
///
/// On success the second value is the `.staging-*` directory when this body
/// wrote a file. The caller decides how long it lives (`StagingDir` for a
/// request). Error paths delete that directory before returning.
pub(crate) async fn parse_multipart_guarded(
    mut multipart: Multipart,
    upload_dir: &Path,
    max_upload_mb: u64,
    max_text_field_kb: usize,
    csrf_session: Option<&str>,
) -> Result<(StrVecMap, Option<Lease>), Response> {
    let max_file_bytes = max_upload_mb.saturating_mul(1024).saturating_mul(1024);
    let max_text_bytes = max_text_field_kb.saturating_mul(1024);

    let mut data: StrVecMap = HashMap::new();
    // Staging dir (under upload_dir → même filesystem, donc le rename de finalize()
    // est atomique). Créé à la demande au premier vrai fichier. La requête qui le
    // reçoit le supprime en partant ; `finalize` en a déjà sorti les fichiers gardés.
    let mut tmp_dir: Option<Lease> = None;
    let mut csrf_checked = csrf_session.is_none();

    // Dossiers laissés par un processus mort avant la fin de sa requête.
    sweep_stale_staging(upload_dir).await;

    loop {
        // An error here (body over the limit, cut connection) ends the request:
        // stopping quietly would hand the form a body cut short.
        let mut field = match multipart.next_field().await {
            Ok(Some(field)) => field,
            Ok(None) => break,
            Err(e) => {
                if let Some(lease) = tmp_dir.take() {
                    lease.discard();
                }
                return Err(
                    (e.status(), t("forms.multipart_stream_error").into_owned()).into_response()
                );
            }
        };
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

            if !csrf_checked {
                return Err((
                    StatusCode::FORBIDDEN,
                    t("csrf.invalid_or_missing").into_owned(),
                )
                    .into_response());
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
                // Claimed before it exists: a sweep running alongside this body
                // must not treat the new directory as abandoned.
                let mut lease = Lease::claim(dir);
                if let Err(err) = tokio::fs::create_dir_all(lease.path()).await {
                    drop(lease);
                    warn!(error = %err, "staging directory was not created");
                    return Err((
                        StatusCode::INTERNAL_SERVER_ERROR,
                        t("forms.upload_dir_error").to_string(),
                    )
                        .into_response());
                }
                if let Err(err) = lease.lock_dir() {
                    warn!(error = %err, "staging directory was not locked");
                    lease.discard();
                    return Err((
                        StatusCode::INTERNAL_SERVER_ERROR,
                        t("forms.upload_dir_error").to_string(),
                    )
                        .into_response());
                }
                tmp_dir = Some(lease);
            }
            let cur_tmp = tmp_dir.as_ref().unwrap().path();

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
                if let Some(lease) = tmp_dir.take() {
                    lease.discard();
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
                Ok(text) => {
                    if let Some(session) = csrf_session
                        && name == CSRF_TOKEN_KEY
                        && crate::forms::extractor::token_matches(&text, session)
                    {
                        csrf_checked = true;
                    }
                    data.entry(name).or_default().push(text);
                }
                Err(e) => {
                    if let Some(lease) = tmp_dir.take() {
                        lease.discard();
                    }
                    return Err(e);
                }
            }
        }
    }

    // Pas de commit ici. Le dossier est rendu à l'appelant : la requête le
    // supprime en se terminant, quel que soit le résultat, et un formulaire qui
    // n'atteint pas `finalize` retire avant ça les fichiers encore dedans.
    // `finalize` est le seul à les déplacer vers la destination servie. Le
    // balayage ne retire que les dossiers qu'aucun processus vivant ne tient.

    Ok((data, tmp_dir))
}

/// Removes `.staging-*` directories no live process holds. A directory this
/// process reserved is skipped, however long the body has been arriving. A
/// directory another process has locked is skipped. Anything else was left by
/// a process that died, and goes now. A symlink is not followed.
async fn sweep_stale_staging(upload_dir: &Path) {
    let mut entries = match tokio::fs::read_dir(upload_dir).await {
        Ok(e) => e,
        Err(_) => return, // upload_dir pas encore créé : rien à purger
    };
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
        if with_live(|live| live.contains(&path)) {
            continue;
        }
        let Ok(meta) = std::fs::symlink_metadata(&path) else {
            continue;
        };
        if !meta.is_dir() || meta.file_type().is_symlink() {
            continue;
        }
        let Ok(file) = File::open(&path) else {
            continue;
        };
        if !try_lock_exclusive(&file) {
            continue;
        }
        // Reserved while we waited for the lock.
        if with_live(|live| live.contains(&path)) {
            continue;
        }
        if let Err(err) = tokio::fs::remove_dir_all(&path).await {
            warn!(dir = %path.display(), error = %err, "staging sweep: remove failed");
        }
    }
}

/// Builds a safe on-disk name `{uuid}.{ext}`. The user-supplied basename is
/// discarded (the UUID kills path traversal / name collisions); the extension is
/// kept only as lowercase ASCII alphanumerics, capped at 10 chars. This strips
/// exotic bytes — notably `:` which on NTFS would open an Alternate Data Stream
/// (`photo.php:zone`) — and normalizes case to match the extension whitelist.
fn sanitize_filename(filename: &str) -> String {
    let ext: String = Path::new(filename)
        .extension()
        .and_then(|e: &std::ffi::OsStr| e.to_str())
        .unwrap_or("")
        .chars()
        .filter(char::is_ascii_alphanumeric)
        .map(|c| c.to_ascii_lowercase())
        .take(10)
        .collect();
    let uuid = Uuid::new_v4().to_string();
    if ext.is_empty() {
        uuid
    } else {
        format!("{uuid}.{ext}")
    }
}

#[cfg(test)]
mod sanitize_tests {
    use super::sanitize_filename;

    fn ext_of(name: &str) -> String {
        // sanitize_filename returns "{uuid}.{ext}" (or just "{uuid}"); grab the ext.
        match name.rsplit_once('.') {
            Some((_, ext)) => ext.to_string(),
            None => String::new(),
        }
    }

    #[test]
    fn strips_ntfs_ads_colon_from_extension() {
        // "photo.php:zone" → extension "php:zone". The security property is that the
        // ':' (which opens an NTFS Alternate Data Stream) is removed; the remaining
        // alphanumerics are concatenated, so "php:zone" → "phpzone" (no ':' left).
        let name = sanitize_filename("photo.php:zone");
        assert!(!name.contains(':'), "colon must be stripped: {name}");
        assert_eq!(ext_of(&name), "phpzone");
    }

    #[test]
    fn lowercases_and_keeps_alnum() {
        assert_eq!(ext_of(&sanitize_filename("IMG.JPolice")), "jpolice");
        assert_eq!(ext_of(&sanitize_filename("archive.PNG")), "png");
    }

    #[test]
    fn drops_exotic_unicode_and_caps_length() {
        // Non-ASCII stripped entirely.
        assert_eq!(ext_of(&sanitize_filename("x.café")), "caf");
        // Capped at 10 chars.
        assert_eq!(
            ext_of(&sanitize_filename("x.abcdefghijklmnop")),
            "abcdefghij"
        );
    }

    #[test]
    fn no_extension_yields_bare_uuid() {
        // No dot → no extension → bare uuid (no trailing dot).
        assert!(!sanitize_filename("noext").contains('.'));
        // Extension made entirely of stripped chars → treated as no extension.
        assert!(!sanitize_filename("weird.:::").contains('.'));
    }

    #[test]
    fn dotfile_has_no_extension() {
        // ".htaccess" has no extension per Path semantics → bare uuid.
        assert!(!sanitize_filename(".htaccess").contains('.'));
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

    /// The upload must NOT be committed under the media_root during parsing:
    /// it stays in staging (`.staging-*`), `finalize` will commit it later, after
    /// CSRF + validation. Secures C1 (no served write before it's been checked).
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

    const SESSION: &str = "a3f1c2d4e5b60718293a4b5c6d7e8f90a3f1c2d4e5b60718293a4b5c6d7e8f90";
    const OTHER: &str = "0000000000000000000000000000000000000000000000000000000000000000";

    /// A body of `(name, filename, content)` parts, in this order.
    fn parts_req(parts: &[(&str, Option<&str>, &str)]) -> Request<Body> {
        let b = "BNDRY";
        let mut body = String::new();
        for (name, filename, content) in parts {
            let disposition = match filename {
                Some(f) => format!("form-data; name=\"{name}\"; filename=\"{f}\""),
                None => format!("form-data; name=\"{name}\""),
            };
            body.push_str(&format!(
                "--{b}\r\nContent-Disposition: {disposition}\r\n\r\n{content}\r\n"
            ));
        }
        body.push_str(&format!("--{b}--\r\n"));
        Request::builder()
            .method("POST")
            .header("content-type", format!("multipart/form-data; boundary={b}"))
            .body(Body::from(body))
            .unwrap()
    }

    /// Parses with the CSRF gate on; the media folder is removed afterwards.
    /// Returns the result and whether anything was written under it.
    async fn gated(parts: &[(&str, Option<&str>, &str)]) -> (Result<StrVecMap, u16>, bool) {
        let media = std::env::temp_dir().join(format!("rq_gate_{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&media).unwrap();
        let mp = Multipart::from_request(parts_req(parts), &())
            .await
            .unwrap();
        let out = parse_multipart_guarded(mp, &media, 10, 64, Some(SESSION))
            .await
            .map(|(data, _staging)| data)
            .map_err(|r| r.status().as_u16());
        let written = std::fs::read_dir(&media).unwrap().next().is_some();
        let _ = std::fs::remove_dir_all(&media);
        (out, written)
    }

    #[tokio::test]
    async fn a_file_is_written_only_after_a_valid_csrf_field() {
        let token = crate::utils::crypto::csrf::mask_csrf_token(SESSION).unwrap();
        let (out, written) =
            gated(&[("csrf_token", None, &token), ("doc", Some("a.pdf"), "x")]).await;
        assert!(out.is_ok() && written, "token first: the file is staged");
    }

    #[tokio::test]
    async fn a_file_before_the_csrf_field_is_refused_unwritten() {
        let token = crate::utils::crypto::csrf::mask_csrf_token(SESSION).unwrap();
        let (out, written) = gated(&[
            ("doc", Some("a.html"), "<script>"),
            ("csrf_token", None, &token),
        ])
        .await;
        assert_eq!(out.err(), Some(403));
        assert!(!written, "nothing on disk");
    }

    #[tokio::test]
    async fn a_wrong_or_missing_token_writes_nothing() {
        let other = crate::utils::crypto::csrf::mask_csrf_token(OTHER).unwrap();
        for parts in [
            vec![
                ("csrf_token", None, other.as_str()),
                ("doc", Some("a.pdf"), "x"),
            ],
            vec![("title", None, "t"), ("doc", Some("a.pdf"), "x")],
        ] {
            let (out, written) = gated(&parts).await;
            assert_eq!(out.err(), Some(403));
            assert!(!written);
        }
    }

    #[tokio::test]
    async fn without_files_the_gate_lets_the_form_through() {
        // No file: nothing to protect here, the token is checked by Prisme.
        let (out, written) = gated(&[("title", None, "t")]).await;
        assert!(out.is_ok() && !written);
    }
}

#[cfg(test)]
mod staging_dir_tests {
    use super::{Lease, StagingDir, sweep_stale_staging};
    use std::os::unix::fs::symlink;
    use std::sync::Arc;

    fn request_dir(path: std::path::PathBuf) -> Arc<StagingDir> {
        let mut lease = Lease::claim(path);
        let _ = lease.lock_dir();
        Arc::new(StagingDir::from_lease(lease))
    }

    #[test]
    fn dropping_the_guard_removes_the_staging_directory_and_nothing_else() {
        let root = std::env::temp_dir().join(format!("rq_hold_{}", uuid::Uuid::new_v4()));
        let staging = root.join(format!(".staging-{}", uuid::Uuid::new_v4()));
        let kept = root.join(format!("kept-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&staging).unwrap();
        std::fs::create_dir_all(&kept).unwrap();
        std::fs::write(staging.join("a.txt"), b"x").unwrap();
        std::fs::write(kept.join("b.txt"), b"y").unwrap();
        let hold = request_dir(staging.clone());
        let cloned = std::sync::Arc::clone(&hold);
        drop(hold);
        assert!(
            staging.join("a.txt").exists(),
            "a copied request still owns the directory"
        );
        drop(cloned);
        {
            let _kept = request_dir(kept.clone());
        }
        assert!(!staging.exists(), "the request's directory is gone");
        assert!(kept.join("b.txt").exists(), "another directory stays");
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn a_symlink_wearing_a_staging_name_is_not_removed() {
        let root = std::env::temp_dir().join(format!("rq_link_{}", uuid::Uuid::new_v4()));
        let secret = root.join("secret");
        std::fs::create_dir_all(&secret).unwrap();
        std::fs::write(secret.join("k.txt"), b"k").unwrap();
        let link = root.join(format!(".staging-{}", uuid::Uuid::new_v4()));
        symlink(&secret, &link).unwrap();
        {
            let _guard = request_dir(link);
        }
        assert!(secret.join("k.txt").exists(), "the target stays");
        let _ = std::fs::remove_dir_all(&root);
    }

    #[tokio::test]
    async fn a_directory_this_process_holds_survives_the_sweep() {
        let root = std::env::temp_dir().join(format!("rq_live_{}", uuid::Uuid::new_v4()));
        let staging = root.join(format!(".staging-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&staging).unwrap();
        std::fs::write(staging.join("a.txt"), b"x").unwrap();
        let hold = request_dir(staging.clone());
        sweep_stale_staging(&root).await;
        assert!(
            staging.join("a.txt").exists(),
            "the upload is still arriving"
        );
        drop(hold);
        assert!(!staging.exists(), "the request ended");
        let _ = std::fs::remove_dir_all(&root);
    }

    #[tokio::test]
    async fn a_directory_no_process_holds_is_removed_at_once() {
        let root = std::env::temp_dir().join(format!("rq_dead_{}", uuid::Uuid::new_v4()));
        let staging = root.join(format!(".staging-{}", uuid::Uuid::new_v4()));
        let kept = root.join("kept");
        std::fs::create_dir_all(&staging).unwrap();
        std::fs::create_dir_all(&kept).unwrap();
        std::fs::write(staging.join("a.txt"), b"x").unwrap();
        sweep_stale_staging(&root).await;
        assert!(!staging.exists(), "a dead process left this");
        assert!(kept.exists(), "anything else stays");
        let _ = std::fs::remove_dir_all(&root);
    }

    #[tokio::test]
    async fn a_symlink_is_not_swept() {
        let root = std::env::temp_dir().join(format!("rq_sweep_link_{}", uuid::Uuid::new_v4()));
        let secret = root.join("secret");
        std::fs::create_dir_all(&secret).unwrap();
        std::fs::write(secret.join("k.txt"), b"k").unwrap();
        let link = root.join(format!(".staging-{}", uuid::Uuid::new_v4()));
        symlink(&secret, &link).unwrap();
        sweep_stale_staging(&root).await;
        assert!(secret.join("k.txt").exists(), "the target stays");
        let _ = std::fs::remove_dir_all(&root);
    }
}
