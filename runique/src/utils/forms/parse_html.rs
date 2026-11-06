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

const DIRECT_UPLOAD_GRACE_PERIOD: std::time::Duration = std::time::Duration::from_secs(60 * 60);

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
    /// Exclusive lock, held for the life of this lease. On Unix this is the
    /// directory itself (`flock`). On Windows it is the `.lock` file inside it
    /// (`LockFileEx`): a directory has no byte range to lock. Another process's
    /// sweep fails to take it and leaves the directory alone. The kernel drops
    /// the lock when this process dies, or when this handle closes.
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
    /// the folder it points at. On Windows the lock is the `.lock` file inside
    /// the directory, and that name is not followed either.
    pub(crate) fn lock_dir(&mut self) -> std::io::Result<()> {
        if self._lock.is_some() {
            return Ok(());
        }
        let meta = std::fs::symlink_metadata(&self.path)?;
        if !meta.is_dir() || meta.file_type().is_symlink() {
            return Ok(());
        }
        let file = open_staging_lock(&self.path)?;
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
        // Close the handle before removing the directory. On Windows the lock
        // file lives inside it, and an open file there makes `remove_dir_all`
        // fail. Closing also releases the kernel lock.
        self._lock.take();
        if self.delete_on_drop {
            remove_staging_dir(&self.path);
        }
    }
}

/// The `.staging-*` directory one request created. The last clone removes it,
/// and only it: the name has to be the directory `parse_multipart` made, and a
/// symlink is left untouched so the removal can't be pointed at another folder.
/// Published files have already been linked out of staging by the form save.
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

/// Opens the object both sides lock. Unix locks the directory. Windows locks
/// `.lock` inside it, creating that file when it is absent. A symlink at that
/// name is refused: opening it would lock whatever it points at.
fn open_staging_lock(dir: &Path) -> std::io::Result<File> {
    #[cfg(unix)]
    {
        File::open(dir)
    }
    #[cfg(windows)]
    {
        let path = dir.join(".lock");
        match std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .create_new(true)
            .open(&path)
        {
            Ok(file) => Ok(file),
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
                open_existing_lock(&path)
            }
            Err(error) => Err(error),
        }
    }
    #[cfg(not(any(unix, windows)))]
    {
        let _ = dir;
        compile_error!("staging directory locks are implemented for Unix and Windows");
    }
}

#[cfg(windows)]
fn open_existing_lock(path: &Path) -> std::io::Result<File> {
    use std::os::windows::fs::OpenOptionsExt;
    // `FILE_FLAG_OPEN_REPARSE_POINT`: the handle is the directory entry itself,
    // not the target of a symlink planted under the name `.lock`.
    const FILE_FLAG_OPEN_REPARSE_POINT: u32 = 0x0020_0000;
    let file = std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .custom_flags(FILE_FLAG_OPEN_REPARSE_POINT)
        .open(path)?;
    if std::fs::symlink_metadata(path).is_ok_and(|meta| meta.file_type().is_symlink()) {
        return Err(std::io::Error::new(
            std::io::ErrorKind::AlreadyExists,
            "staging lock is a symlink",
        ));
    }
    Ok(file)
}

#[cfg(unix)]
fn lock_exclusive(file: &File) -> std::io::Result<()> {
    use std::os::unix::io::AsRawFd;
    // Safety: `file` is an open descriptor. `LOCK_EX` blocks until this process
    // is the only holder; the kernel releases it when the descriptor closes.
    let rc = unsafe { libc::flock(file.as_raw_fd(), libc::LOCK_EX) };
    if rc == 0 {
        Ok(())
    } else {
        Err(std::io::Error::last_os_error())
    }
}

#[cfg(windows)]
fn lock_exclusive(file: &File) -> std::io::Result<()> {
    lock_file(file, LOCKFILE_EXCLUSIVE_LOCK)
}

/// `false` when another live process already holds the directory.
#[cfg(unix)]
fn try_lock_exclusive(file: &File) -> bool {
    use std::os::unix::io::AsRawFd;
    // Safety: same descriptor rules as `lock_exclusive`. `LOCK_NB` returns at once.
    unsafe { libc::flock(file.as_raw_fd(), libc::LOCK_EX | libc::LOCK_NB) == 0 }
}

#[cfg(windows)]
fn try_lock_exclusive(file: &File) -> bool {
    lock_file(file, LOCKFILE_EXCLUSIVE_LOCK | LOCKFILE_FAIL_IMMEDIATELY).is_ok()
}

#[cfg(windows)]
const LOCKFILE_FAIL_IMMEDIATELY: u32 = 0x0000_0001;
#[cfg(windows)]
const LOCKFILE_EXCLUSIVE_LOCK: u32 = 0x0000_0002;

#[cfg(windows)]
#[link(name = "kernel32")]
unsafe extern "system" {
    fn LockFileEx(
        file: *mut std::ffi::c_void,
        flags: u32,
        reserved: u32,
        bytes_low: u32,
        bytes_high: u32,
        overlapped: *mut std::ffi::c_void,
    ) -> i32;
}

#[cfg(windows)]
fn lock_file(file: &File, flags: u32) -> std::io::Result<()> {
    use std::os::windows::io::AsRawHandle;
    // Synchronous handle: a zeroed `OVERLAPPED` locks from offset 0. One byte is
    // enough, provided every caller locks that same byte. The buffer is larger
    // than `OVERLAPPED` on both 32-bit and 64-bit so the stack slot stays valid
    // for the call.
    let mut overlapped = [0u64; 4];
    // Safety: `file` is an open handle. `overlapped` is live and aligned for the
    // duration of the call. The kernel releases the lock when the handle closes.
    let rc = unsafe {
        LockFileEx(
            file.as_raw_handle(),
            flags,
            0,
            1,
            0,
            overlapped.as_mut_ptr().cast(),
        )
    };
    if rc != 0 {
        Ok(())
    } else {
        Err(std::io::Error::last_os_error())
    }
}

/// Reads a multipart body: text fields as they are, files written to a
/// `.staging-*` folder under `upload_dir` (their path as the value), each held
/// to `max_upload_mb`. No CSRF check and no lease: only the tests use it, the
/// request pipeline uses `parse_multipart_guarded`.
#[cfg(test)]
pub(crate) async fn parse_multipart(
    multipart: Multipart,
    upload_dir: &Path,
    max_upload_mb: u64,
    max_text_field_kb: usize,
) -> Result<StrVecMap, Response> {
    // The request pipeline holds its staging lease until request completion.
    // Direct callers own the returned file paths; the sweeper grants recent
    // files a bounded grace period because this API cannot return that lease.
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
    // Staging dir under upload_dir so publication can atomically link files on
    // the same filesystem. Created on the first real file and removed when the
    // owning request ends.
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
                // tokio's write_all returns once the bytes are handed to a
                // background task: without this, validation can read the size
                // or content of a partial file, and a failed write goes unseen.
                file.flush().await.map_err(|_| {
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        t("forms.file_write_error").to_string(),
                    )
                        .into_response()
                })?;
                Ok(())
            }
            .await;

            if let Err(e) = stream_result {
                if let Some(lease) = tmp_dir.take() {
                    lease.discard();
                }
                return Err(e);
            }

            // Staging path only: the form publishes it between its database
            // write and the commit. No file is served before CSRF and validation pass.
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

    // Pas de publication ici. Le dossier est rendu à l'appelant : la requête le
    // supprime en se terminant, quel que soit le résultat, et un formulaire qui
    // n'atteint pas `finalize` retire avant ça les fichiers encore dedans.
    // `publish_uploads` est l'étape qui les rend servis, juste avant l'écriture DB. Le
    // balayage ne retire que les dossiers qu'aucun processus vivant ne tient.

    Ok((data, tmp_dir))
}

/// Removes abandoned `.staging-*` directories. Active leases are protected by
/// their lock; direct parser callers keep recent files for a bounded grace
/// period. A symlink is not followed.
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
        let Ok(file) = open_staging_lock(&path) else {
            continue;
        };
        if !try_lock_exclusive(&file) {
            continue;
        }
        // Reserved while we waited for the lock.
        if with_live(|live| live.contains(&path)) {
            continue;
        }
        if has_recent_staged_file(&path).await {
            continue;
        }
        // The handle has to close before the directory goes. On Windows it is
        // `.lock`, which sits inside the directory.
        drop(file);
        if let Err(err) = tokio::fs::remove_dir_all(&path).await {
            warn!(dir = %path.display(), error = %err, "staging sweep: remove failed");
        }
    }
}

async fn has_recent_staged_file(dir: &Path) -> bool {
    let mut entries = match tokio::fs::read_dir(dir).await {
        Ok(entries) => entries,
        Err(_) => return true,
    };
    loop {
        let entry = match entries.next_entry().await {
            Ok(Some(entry)) => entry,
            Ok(None) => return false,
            Err(_) => return true,
        };
        if entry.file_name() == ".lock" {
            continue;
        }
        let Ok(file_type) = entry.file_type().await else {
            return true;
        };
        if !file_type.is_file() {
            continue;
        }
        let Ok(metadata) = entry.metadata().await else {
            return true;
        };
        match metadata.modified() {
            Ok(modified)
                if modified
                    .elapsed()
                    .is_ok_and(|age| age < DIRECT_UPLOAD_GRACE_PERIOD) =>
            {
                return true;
            }
            Ok(_) => {}
            Err(_) => return true,
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
    #[cfg(unix)]
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

    #[cfg(unix)]
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
    async fn a_stale_directory_no_process_holds_is_removed() {
        let root = std::env::temp_dir().join(format!("rq_dead_{}", uuid::Uuid::new_v4()));
        let staging = root.join(format!(".staging-{}", uuid::Uuid::new_v4()));
        let kept = root.join("kept");
        std::fs::create_dir_all(&staging).unwrap();
        std::fs::create_dir_all(&kept).unwrap();
        let staged_file = staging.join("a.txt");
        std::fs::write(&staged_file, b"x").unwrap();
        std::fs::File::options()
            .write(true)
            .open(&staged_file)
            .unwrap()
            .set_times(std::fs::FileTimes::new().set_modified(std::time::UNIX_EPOCH))
            .unwrap();
        sweep_stale_staging(&root).await;
        assert!(!staging.exists(), "a dead process left this");
        assert!(kept.exists(), "anything else stays");
        let _ = std::fs::remove_dir_all(&root);
    }

    #[cfg(unix)]
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

/// `parse_multipart` itself: text fields, files, size limits, the sweep.
/// Moved from `tests/utils/test_parse_html.rs` when it became `pub(crate)`.
#[cfg(test)]
mod direct_parse_tests {
    use super::parse_multipart;
    use axum::{
        Router,
        body::Body,
        extract::Multipart,
        http::{Request, StatusCode, header},
        response::IntoResponse,
        routing::post,
    };
    use tower::ServiceExt;

    // ── Handler de test ───────────────────────────────────────────────────────────

    async fn multipart_handler(multipart: Multipart) -> impl IntoResponse {
        let upload_dir = std::env::temp_dir().join("runique_test_upload");
        match parse_multipart(multipart, &upload_dir, 100, 1024).await {
            Ok(data) => {
                // Retourne le nombre de champs parsés dans le body
                let count = data.len().to_string();
                (StatusCode::OK, count)
            }
            Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "error".to_string()),
        }
    }

    fn multipart_app() -> Router {
        Router::new().route("/upload", post(multipart_handler))
    }

    // ── Helper : construction d'une requête multipart texte ──────────────────────

    fn multipart_text_request(fields: &[(&str, &str)]) -> Request<Body> {
        let boundary = "----boundary123456";
        let mut body = String::new();
        for (name, value) in fields {
            body.push_str(&format!(
                "--{boundary}\r\nContent-Disposition: form-data; name=\"{name}\"\r\n\r\n{value}\r\n"
            ));
        }
        body.push_str(&format!("--{boundary}--\r\n"));

        Request::builder()
            .method("POST")
            .uri("/upload")
            .header(
                header::CONTENT_TYPE,
                format!("multipart/form-data; boundary={boundary}"),
            )
            .body(Body::from(body))
            .unwrap()
    }

    // ── Tests — champs texte ─────────────────────────────────────────────────────

    #[tokio::test]
    async fn test_parse_multipart_champ_texte_simple() {
        let req = multipart_text_request(&[("nom", "alice")]);
        let resp = multipart_app().oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_parse_multipart_plusieurs_champs() {
        let req = multipart_text_request(&[("nom", "alice"), ("age", "30"), ("ville", "Paris")]);
        let resp = multipart_app().oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_parse_multipart_body_vide() {
        let boundary = "----boundary_empty";
        let body = format!("--{boundary}--\r\n");
        let req = Request::builder()
            .method("POST")
            .uri("/upload")
            .header(
                header::CONTENT_TYPE,
                format!("multipart/form-data; boundary={boundary}"),
            )
            .body(Body::from(body))
            .unwrap();
        let resp = multipart_app().oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
    }

    // ── Tests — upload de fichier ─────────────────────────────────────────────────

    fn multipart_file_request(field_name: &str, filename: &str, content: &str) -> Request<Body> {
        let boundary = "----fileboundary789";
        let body = format!(
            "--{boundary}\r\nContent-Disposition: form-data; name=\"{field_name}\"; filename=\"{filename}\"\r\nContent-Type: text/plain\r\n\r\n{content}\r\n--{boundary}--\r\n"
        );
        Request::builder()
            .method("POST")
            .uri("/upload")
            .header(
                header::CONTENT_TYPE,
                format!("multipart/form-data; boundary={boundary}"),
            )
            .body(Body::from(body))
            .unwrap()
    }

    #[tokio::test]
    async fn test_parse_multipart_upload_fichier() {
        let req = multipart_file_request("fichier", "test.txt", "contenu du fichier");
        let resp = multipart_app().oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_parse_multipart_upload_sans_extension() {
        let req = multipart_file_request("fichier", "noext", "données sans extension");
        let resp = multipart_app().oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_parse_multipart_mix_texte_et_fichier() {
        let boundary = "----mixboundary";
        let body = format!(
            "--{boundary}\r\nContent-Disposition: form-data; name=\"nom\"\r\n\r\nbob\r\n\
             --{boundary}\r\nContent-Disposition: form-data; name=\"doc\"; filename=\"doc.pdf\"\r\nContent-Type: application/pdf\r\n\r\n%PDF-1.4\r\n\
             --{boundary}--\r\n"
        );
        let req = Request::builder()
            .method("POST")
            .uri("/upload")
            .header(
                header::CONTENT_TYPE,
                format!("multipart/form-data; boundary={boundary}"),
            )
            .body(Body::from(body))
            .unwrap();
        let resp = multipart_app().oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
    }

    // ── Written from cargo-mutants survivors (2026-10-08) ─────────────────────────

    /// Parses a multipart request into a fresh `dir`, with 1 MB per file and 1 KB
    /// per text field; returns the status.
    async fn parse_into(dir: std::path::PathBuf, req: Request<Body>) -> StatusCode {
        let app = Router::new().route(
            "/upload",
            post(move |multipart: Multipart| {
                let dir = dir.clone();
                async move {
                    match parse_multipart(multipart, &dir, 1, 1).await {
                        Ok(_) => StatusCode::OK,
                        Err(resp) => resp.status(),
                    }
                }
            }),
        );
        app.oneshot(req).await.unwrap().status()
    }

    fn temp_upload_dir() -> std::path::PathBuf {
        std::env::temp_dir().join(format!("rq_parse_{}", uuid::Uuid::new_v4()))
    }

    /// Exactly at a limit is accepted; one byte over is refused (parse_html.rs:108, 154).
    #[tokio::test]
    async fn upload_limits_take_the_limit_itself() {
        let dir = temp_upload_dir();
        let text = |n: usize| multipart_text_request(&[("bio", &"a".repeat(n))]);
        assert_eq!(parse_into(dir.clone(), text(1024)).await, StatusCode::OK);
        assert_eq!(
            parse_into(dir.clone(), text(1025)).await,
            StatusCode::PAYLOAD_TOO_LARGE
        );

        let file = |n: usize| multipart_file_request("doc", "a.txt", &"b".repeat(n));
        assert_eq!(
            parse_into(dir.clone(), file(1024 * 1024)).await,
            StatusCode::OK
        );
        assert_eq!(
            parse_into(dir.clone(), file(1024 * 1024 + 1)).await,
            StatusCode::PAYLOAD_TOO_LARGE
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[tokio::test]
    async fn direct_multipart_file_remains_available_after_an_unrelated_parse() {
        use axum::extract::Multipart;

        let dir = temp_upload_dir();
        let upload_dir = dir.clone();
        let app = Router::new().route(
            "/upload",
            post(move |multipart: Multipart| {
                let upload_dir = upload_dir.clone();
                async move {
                    let parsed = parse_multipart(multipart, &upload_dir, 1, 1)
                        .await
                        .expect("multipart parsed");
                    parsed["doc"][0].clone()
                }
            }),
        );
        let response = app
            .oneshot(multipart_file_request("doc", "report.pdf", "%PDF-1.4"))
            .await
            .unwrap();
        let path = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let path = String::from_utf8(path.to_vec()).unwrap();

        assert!(
            std::path::Path::new(&path).exists(),
            "the returned upload exists"
        );
        assert_eq!(
            parse_into(dir.clone(), multipart_text_request(&[("note", "ok")])).await,
            StatusCode::OK
        );
        let still_exists = std::path::Path::new(&path).exists();
        let _ = std::fs::remove_dir_all(&dir);

        assert!(
            still_exists,
            "an unrelated parse must not sweep a file path still owned by the direct caller"
        );
    }

    /// A parse removes every `.staging-*` directory no live process holds, however
    /// recent it is. Any other folder stays (`sweep_stale_staging`).
    #[tokio::test]
    async fn stale_staging_folders_are_swept_and_nothing_else() {
        let dir = temp_upload_dir();
        let two_hours_ago = std::time::SystemTime::now() - std::time::Duration::from_secs(7200);
        for name in [".staging-old", ".staging-new", "kept-old"] {
            std::fs::create_dir_all(dir.join(name)).unwrap();
        }
        for name in [".staging-old", "kept-old"] {
            std::fs::File::open(dir.join(name))
                .unwrap()
                .set_modified(two_hours_ago)
                .unwrap();
        }

        let status = parse_into(dir.clone(), multipart_text_request(&[("nom", "x")])).await;
        let (old, new, other) = (
            dir.join(".staging-old").exists(),
            dir.join(".staging-new").exists(),
            dir.join("kept-old").exists(),
        );
        let _ = std::fs::remove_dir_all(&dir);
        assert_eq!(status, StatusCode::OK);
        assert!(!old, "an unclaimed staging folder is removed");
        assert!(!new, "a fresh unclaimed staging folder is removed too");
        assert!(other, "anything that isn't staging is never touched");
    }
}

/// The upload chain from `parse_multipart` on: fill → validate → publish, and
/// what a file field accepts as its value. Moved from
/// `tests/formulaire/test_file_field_path_guard.rs` when `parse_multipart`
/// became `pub(crate)`.
#[cfg(test)]
mod upload_chain_tests {
    use super::parse_multipart;
    use crate::forms::Forms;
    use crate::forms::fields::FileField;
    use axum::{
        Router,
        body::Body,
        extract::Multipart,
        http::{Request, header},
        routing::post,
    };
    use tower::ServiceExt;

    /// Stages the request's files the way Prisme does, then runs a form with
    /// one `doc` file field; answers with the value the field ends up with.
    async fn handler(multipart: Multipart) -> String {
        let media = std::env::var("MEDIA_ROOT").expect("MEDIA_ROOT set by the test");
        let parsed = parse_multipart(multipart, std::path::Path::new(&media), 10, 64)
            .await
            .expect("multipart parsed");
        let data = parsed.into_iter().map(|(k, v)| (k, v.join(","))).collect();
        let mut form = Forms::new("csrf");
        form.field(&FileField::document("doc"));
        form.fill(&data, axum::http::Method::POST);
        if !form.is_valid().await.unwrap_or(false) {
            return "invalid".into();
        }
        form.finalize().await.expect("finalize");
        form.publish_uploads().await.expect("publish");
        form.complete_uploads().await;
        form.fields["doc"].value().to_string()
    }

    fn request(parts: &[(&str, Option<&str>, &str)]) -> Request<Body> {
        let boundary = "----rqboundary";
        let mut body = String::new();
        for (name, filename, content) in parts {
            let disposition = match filename {
                Some(f) => format!("form-data; name=\"{name}\"; filename=\"{f}\""),
                None => format!("form-data; name=\"{name}\""),
            };
            body.push_str(&format!(
                "--{boundary}\r\nContent-Disposition: {disposition}\r\n\r\n{content}\r\n"
            ));
        }
        body.push_str(&format!("--{boundary}--\r\n"));
        Request::builder()
            .method("POST")
            .uri("/upload")
            .header(
                header::CONTENT_TYPE,
                format!("multipart/form-data; boundary={boundary}"),
            )
            .body(Body::from(body))
            .unwrap()
    }

    async fn send(req: Request<Body>) -> String {
        let app = Router::new().route("/upload", post(handler));
        let resp = app.oneshot(req).await.expect("response");
        let bytes = axum::body::to_bytes(resp.into_body(), usize::MAX)
            .await
            .expect("body");
        String::from_utf8(bytes.to_vec()).expect("utf-8")
    }

    fn media_root() -> std::path::PathBuf {
        let media = std::env::temp_dir().join(format!("rq_media_{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&media).unwrap();
        set_env("MEDIA_ROOT", media.to_str().unwrap());
        media
    }

    #[tokio::test]
    async fn a_real_upload_is_committed_to_media_root() {
        let _g = crate::config::static_files::MEDIA_ENV_LOCK.lock().await;
        let media = media_root();
        let value = send(request(&[("doc", Some("report.pdf"), "%PDF-1.4")])).await;
        // Staging renames the upload (random name, extension kept).
        assert!(value.ends_with(".pdf") && !value.contains('/'), "{value}");
        assert!(media.join(&value).exists(), "committed under MEDIA_ROOT");
        crate::config::static_files::restore_media_root();
        let _ = std::fs::remove_dir_all(&media);
    }

    #[tokio::test]
    async fn a_path_sent_as_text_is_ignored() {
        let _g = crate::config::static_files::MEDIA_ENV_LOCK.lock().await;
        let media = media_root();
        let outside = std::env::temp_dir().join(format!("rq_out_{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&outside).unwrap();
        let victim = outside.join("secret.pdf");
        std::fs::write(&victim, b"server file").unwrap();
        let path = victim.display().to_string();

        // Alone, then next to a real upload under the same name.
        let alone = send(request(&[("doc", None, &path)])).await;
        let mixed = send(request(&[
            ("doc", Some("report.pdf"), "%PDF-1.4"),
            ("doc", None, &path),
        ]))
        .await;

        assert_eq!(alone, "", "no upload, nothing kept");
        assert_eq!(mixed, "", "one path that isn't an upload rejects the value");
        assert!(victim.exists(), "the server file is untouched");
        assert!(
            !media.join("secret.pdf").exists(),
            "and never moved into media"
        );
        crate::config::static_files::restore_media_root();
        let _ = std::fs::remove_dir_all(&media);
        let _ = std::fs::remove_dir_all(&outside);
    }

    /// Parses like Prisme, then fills a form whose `title` is a text field.
    async fn text_title(multipart: Multipart) -> String {
        let media = std::env::var("MEDIA_ROOT").expect("MEDIA_ROOT set by the test");
        let parsed = parse_multipart(multipart, std::path::Path::new(&media), 10, 64)
            .await
            .expect("multipart parsed");
        let data = parsed.into_iter().map(|(k, v)| (k, v.join(","))).collect();
        let mut form = Forms::new("csrf");
        form.field(&crate::forms::fields::TextField::text("title"));
        form.fill(&data, axum::http::Method::POST);
        form.fields["title"].value().to_string()
    }

    // A file part sent under a text field's name: the field must not take the
    // staged path as its value, or a re-rendered form shows where the upload
    // waits (and the file was reachable there).
    #[tokio::test]
    async fn a_text_field_never_shows_a_staged_path() {
        let _g = crate::config::static_files::MEDIA_ENV_LOCK.lock().await;
        let media = media_root();
        let app = Router::new().route("/upload", post(text_title));
        let mut values = Vec::new();
        for req in [
            request(&[("title", Some("page.html"), "<script>")]),
            request(&[("title", None, "Bonjour")]),
        ] {
            let resp = app.clone().oneshot(req).await.expect("response");
            let bytes = axum::body::to_bytes(resp.into_body(), usize::MAX)
                .await
                .expect("body");
            values.push(String::from_utf8(bytes.to_vec()).expect("utf-8"));
        }
        let (file, text) = (values[0].clone(), values[1].clone());

        assert_eq!(file, "", "the staged path is dropped");
        assert_eq!(text, "Bonjour", "a text value still is taken");
        crate::config::static_files::restore_media_root();
        let _ = std::fs::remove_dir_all(&media);
    }

    #[tokio::test]
    async fn upload_to_cannot_escape_media_root() {
        let _g = crate::config::static_files::MEDIA_ENV_LOCK.lock().await;
        use crate::forms::base::FormField;

        let media = media_root();
        let escaped_name = format!("{}_escaped", media.file_name().unwrap().to_string_lossy());
        let escaped_dir = media.parent().unwrap().join(&escaped_name);
        let staged_dir = media.join(".staging-upload-to-guard");
        std::fs::create_dir_all(&staged_dir).unwrap();
        let staged_file = staged_dir.join("report.pdf");
        std::fs::write(&staged_file, b"%PDF-1.4").unwrap();

        let mut field = FileField::document("doc").upload_to(format!("../{escaped_name}"));
        field.set_value(staged_file.to_string_lossy().to_string().as_ref());
        let result = field.finalize().await;
        let staged_remains = staged_file.exists();
        let escaped_file_exists = escaped_dir.join("report.pdf").exists();

        let _ = std::fs::remove_dir_all(&media);
        let _ = std::fs::remove_dir_all(&escaped_dir);
        crate::config::static_files::restore_media_root();

        assert!(
            result.is_err(),
            "an upload destination outside MEDIA_ROOT is refused"
        );
        assert!(
            staged_remains,
            "the staged upload is not moved outside the root"
        );
        assert!(!escaped_file_exists);
    }

    fn set_env(key: &str, val: &str) {
        // SAFETY: the tests touching MEDIA_ROOT hold `MEDIA_ENV_LOCK`.
        unsafe { std::env::set_var(key, val) }
    }
}
