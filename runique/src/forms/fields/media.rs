//! `UploadMedia`: the upload mechanism every field taking files shares —
//! what it accepts, the staged files waiting to move, the ones moved, and the
//! value they replace. `MediaField`: images, audio and video, each checked by
//! its content.
use super::file::{
    AllowedExtensions, FileSize, FileUploadConfig, IntoUploadPath, checked_upload_relative,
    cleanup_files, ensure_safe_upload_dir, is_staged_upload, is_valid_image_content,
    parse_file_list, read_image_dimensions,
};
use crate::config::static_files::resolve_media_root;
use crate::forms::base::{CommonFieldConfig, FieldConfig, FormField};
use crate::utils::aliases::ATera;
use crate::utils::trad::{t, tf};
use std::path::{Path, PathBuf};

/// A staged upload and where `publish` moves it.
#[derive(Clone, Debug)]
struct PendingUpload {
    staged: PathBuf,
    destination: PathBuf,
}

/// The upload mechanism shared by every field that takes files: what it
/// accepts (extensions, size, count), the staged files waiting to move, the
/// ones moved, and the value they replace. Moving a file happens between the
/// row's write and its commit (`publish`), and is taken back if that write
/// doesn't commit (`rollback`).
#[derive(Clone, Debug)]
pub struct UploadMedia {
    pub allowed_extensions: AllowedExtensions,
    pub upload_config: FileUploadConfig,
    pub model_max_size: Option<u64>,
    pub max_files: Option<usize>,
    pub prev_value: Option<String>,
    pending_uploads: Vec<PendingUpload>,
    published_uploads: Vec<PathBuf>,
}

/// What [`UploadMedia::check`] found in a field's value.
pub(crate) enum UploadCheck {
    /// Required and empty: the message to show.
    Required(String),
    /// Nothing sent, and nothing required.
    Empty,
    /// Refused (count, extension, size): the staged files are removed, the
    /// message to show.
    Refused(String),
    /// Every file passed: their paths.
    Files(Vec<String>),
}

impl UploadMedia {
    pub fn new(allowed_extensions: AllowedExtensions) -> Self {
        Self {
            allowed_extensions,
            upload_config: FileUploadConfig::default(),
            model_max_size: None,
            max_files: None,
            prev_value: None,
            pending_uploads: Vec::new(),
            published_uploads: Vec::new(),
        }
    }

    pub(crate) fn set_max_size(&mut self, size: FileSize) {
        self.model_max_size = Some(size.as_bytes());
        self.upload_config = std::mem::take(&mut self.upload_config).max_size(size);
    }

    /// Overrides the effective max_size at the form level.
    /// Returns Err if the requested size exceeds the model-defined ceiling.
    pub(crate) fn apply_max_size_bounded(&mut self, size: FileSize) -> Result<(), String> {
        let bytes = size.as_bytes();
        if let Some(limit) = self.model_max_size
            && bytes > limit
        {
            let req_mb = bytes as f64 / (1024.0 * 1024.0);
            let lim_mb = limit as f64 / (1024.0 * 1024.0);
            return Err(format!(
                "max_size override ({:.1}MB) dépasse la limite du modèle ({:.1}MB)",
                req_mb, lim_mb
            ));
        }
        self.upload_config.max_size = Some(bytes);
        Ok(())
    }

    pub(crate) fn cap_max_size(&mut self, bytes: u64) {
        if self.upload_config.max_size.is_none_or(|s| s > bytes) {
            self.upload_config.max_size = Some(bytes);
        }
    }

    /// Sets `value` on `base`, remembering the stored value it replaces.
    pub(crate) fn track_value(&mut self, base: &mut FieldConfig, value: &str) {
        let current = base.value.trim().to_string();
        // A staged path is not a file already stored. Remembering it would make
        // a later `finalize` try to delete it as the previous media file.
        let stored = parse_file_list(&current)
            .iter()
            .any(|p| !is_staged_upload(p));
        if !current.is_empty() && current != value.trim() && stored {
            self.prev_value = Some(current);
        }
        base.value = value.to_string();
    }

    /// Presence, count, extension and size of the files in `base`'s value.
    pub(crate) async fn check(&self, base: &FieldConfig) -> UploadCheck {
        let val = base.value.trim();

        // 1. Presence validation
        if base.is_required.choice && val.is_empty() {
            let msg = base
                .is_required
                .message
                .clone()
                .unwrap_or_else(|| t("forms.file_required").to_string());
            return UploadCheck::Required(msg);
        }

        if val.is_empty() {
            return UploadCheck::Empty;
        }

        let files: Vec<String> = parse_file_list(val)
            .into_iter()
            .map(|s| s.to_string())
            .collect();

        // 2. File count validation
        if let Some(max) = self.max_files
            && files.len() > max
        {
            cleanup_files(&files).await;
            return UploadCheck::Refused(tf("forms.file_max_count", &[&max]));
        }

        // 3. Extensions + size validation
        for filename in &files {
            if !self.allowed_extensions.is_allowed(filename) {
                let exts = self.allowed_extensions.extensions.join(", ");
                cleanup_files(&files).await;
                return UploadCheck::Refused(tf(
                    "forms.file_extension_blocked",
                    &[filename.as_str(), exts.as_str()],
                ));
            }
            // A path already stored (`plats/x.png`, no file at that path, kept
            // as is by `finalize`) has nothing to measure; a real file or an
            // upload in staging always does.
            let to_measure =
                Path::new(filename.as_str()).exists() || filename.contains(".staging-");
            if let Some(max_bytes) = self.upload_config.max_size
                && to_measure
            {
                // Refused when in doubt: an unreadable size would skip the limit.
                let file_size = match tokio::fs::metadata(filename).await {
                    Ok(metadata) => metadata.len(),
                    Err(e) => {
                        tracing::warn!(field = %base.name, error = %e, "file field: upload size unreadable, refused");
                        cleanup_files(&files).await;
                        return UploadCheck::Refused(t("forms.file_unreadable").to_string());
                    }
                };
                if file_size > max_bytes {
                    let size_mb = file_size as f64 / (1024.0 * 1024.0);
                    let max_mb = max_bytes as f64 / (1024.0 * 1024.0);
                    let size_str = format!("{:.1}", size_mb);
                    let max_mb_str = format!("{:.1}", max_mb);
                    cleanup_files(&files).await;
                    return UploadCheck::Refused(tf(
                        "forms.file_too_large",
                        &[filename.as_str(), size_str.as_str(), max_mb_str.as_str()],
                    ));
                }
            }
        }

        UploadCheck::Files(files)
    }

    pub(crate) async fn finalize(&mut self, base: &mut FieldConfig) -> Result<(), String> {
        let val = base.value.trim().to_string();
        if val.is_empty() {
            return Ok(());
        }

        self.pending_uploads.clear();
        let upload_rel_clean = match &self.upload_config.upload_to {
            Some(f) => checked_upload_relative(&f(&base.name))?,
            None => String::new(),
        };
        let media_root = PathBuf::from(resolve_media_root());
        let dest_dir = media_root.join(&upload_rel_clean);

        // Construit le chemin relatif stocké ("filename" en racine, sinon "rel/filename").
        let to_rel = |filename: &str| -> String {
            if upload_rel_clean.is_empty() {
                filename.to_string()
            } else {
                format!("{}/{}", upload_rel_clean, filename)
            }
        };

        let files: Vec<String> = val
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
        let mut new_paths = Vec::new();
        for file_path in &files {
            let src = Path::new(file_path.as_str());

            if src.parent() == Some(dest_dir.as_path()) {
                let filename = src
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or(file_path.as_str());
                new_paths.push(to_rel(filename));
                continue;
            }

            // Value is already a relative stored path (no physical file at that path) — keep as-is
            if !src.exists() {
                new_paths.push(file_path.clone());
                continue;
            }

            // Only an upload is ever moved: any other existing file stays where it is.
            if !is_staged_upload(file_path) {
                return Err(format!("'{file_path}' is not an upload"));
            }

            let filename = src
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or(file_path.as_str());
            let canonical_root = tokio::fs::canonicalize(&media_root)
                .await
                .map_err(|e| format!("media root '{}': {e}", media_root.display()))?;
            self.pending_uploads.push(PendingUpload {
                staged: src.to_path_buf(),
                destination: canonical_root.join(&upload_rel_clean).join(filename),
            });

            new_paths.push(to_rel(filename));
        }

        base.value = new_paths.join(",");
        Ok(())
    }

    pub(crate) async fn publish(&mut self) -> Result<(), String> {
        if self.pending_uploads.is_empty() {
            return Ok(());
        }

        let media_root = PathBuf::from(resolve_media_root());
        let canonical_root = tokio::fs::canonicalize(&media_root)
            .await
            .map_err(|e| format!("media root '{}': {e}", media_root.display()))?;

        for upload in &self.pending_uploads {
            if !is_staged_upload(&upload.staged.to_string_lossy()) {
                return Err(format!(
                    "'{}' is no longer a staged upload",
                    upload.staged.display()
                ));
            }
            let metadata = tokio::fs::symlink_metadata(&upload.staged)
                .await
                .map_err(|e| format!("staged upload '{}': {e}", upload.staged.display()))?;
            if !metadata.file_type().is_file() {
                return Err(format!(
                    "'{}' is not a regular staged file",
                    upload.staged.display()
                ));
            }

            let parent = upload.destination.parent().ok_or_else(|| {
                format!(
                    "invalid upload destination '{}'",
                    upload.destination.display()
                )
            })?;
            let safe_parent = ensure_safe_upload_dir(&canonical_root, parent).await?;
            let filename = upload.destination.file_name().ok_or_else(|| {
                format!(
                    "invalid upload destination '{}'",
                    upload.destination.display()
                )
            })?;
            let destination = safe_parent.join(filename);

            // Staging is under MEDIA_ROOT, so a hard link publishes the complete
            // file atomically and fails rather than overwriting an existing file.
            tokio::fs::hard_link(&upload.staged, &destination)
                .await
                .map_err(|e| format!("publish '{}': {e}", destination.display()))?;
            self.published_uploads.push(destination);
            if let Err(error) = tokio::fs::remove_file(&upload.staged).await {
                return Err(format!(
                    "remove staged upload '{}': {error}",
                    upload.staged.display()
                ));
            }
        }

        Ok(())
    }

    pub(crate) async fn rollback(&mut self, base: &mut FieldConfig) {
        for path in self.published_uploads.drain(..) {
            if let Err(error) = tokio::fs::remove_file(&path).await
                && error.kind() != std::io::ErrorKind::NotFound
            {
                tracing::error!(path = %path.display(), error = %error, "published upload rollback failed");
            }
        }
        if !self.pending_uploads.is_empty() {
            base.value = self.prev_value.clone().unwrap_or_default();
            self.pending_uploads.clear();
        }
    }

    pub(crate) fn complete(&mut self) {
        self.published_uploads.clear();
        self.pending_uploads.clear();
        self.prev_value = None;
    }
}

/// What a [`MediaField`] takes. Each kind checks the file's content — its
/// signature — not only its extension: a file renamed `.mp4` is refused.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
pub enum MediaKind {
    Image,
    Audio,
    Video,
}

/// Upload field for images, audio and video: [`MediaField::image`],
/// [`::audio`](MediaField::audio), [`::video`](MediaField::video). Shares the
/// upload mechanism of every file field ([`UploadMedia`]); on top, checks the
/// content of each file — an image's signature and dimensions, an audio or
/// video file's signature. Documents and other files: [`FileField`](super::FileField).
#[derive(Clone, Debug)]
pub struct MediaField {
    pub base: FieldConfig,
    pub kind: MediaKind,
    pub upload: UploadMedia,
    pub max_width: Option<u32>,
    pub max_height: Option<u32>,
}

impl CommonFieldConfig for MediaField {
    fn get_field_config(&self) -> &FieldConfig {
        &self.base
    }

    fn get_field_config_mut(&mut self) -> &mut FieldConfig {
        &mut self.base
    }
}

impl MediaField {
    fn create(name: &str, kind: MediaKind) -> Self {
        let extensions = match kind {
            MediaKind::Image => AllowedExtensions::images(),
            MediaKind::Audio => AllowedExtensions::audio(),
            MediaKind::Video => AllowedExtensions::video(),
        };
        Self {
            base: FieldConfig::new(name, "file", "base_file.html"),
            kind,
            upload: UploadMedia::new(extensions),
            max_width: None,
            max_height: None,
        }
    }

    /// Image: jpg, jpeg, png, gif, webp, avif. Checks the signature, and the
    /// dimensions with [`max_dimensions`](Self::max_dimensions). SVG refused.
    pub fn image(name: &str) -> Self {
        Self::create(name, MediaKind::Image)
    }

    /// Audio: mp3, ogg, oga, wav, m4a, flac, aac. Checks the signature.
    pub fn audio(name: &str) -> Self {
        Self::create(name, MediaKind::Audio)
    }

    /// Video: mp4, webm, mov, ogv. Checks the signature.
    pub fn video(name: &str) -> Self {
        Self::create(name, MediaKind::Video)
    }

    /// Overrides the auto-generated label.
    pub fn label(mut self, label: &str) -> Self {
        self.base.label = label.to_string();
        self
    }

    /// Destination directory, relative to `MEDIA_ROOT` (`"podcasts"` →
    /// `{MEDIA_ROOT}/podcasts/`).
    pub fn upload_to(mut self, path: impl IntoUploadPath) -> Self {
        self.upload.upload_config =
            std::mem::take(&mut self.upload.upload_config).upload_to(path.into_upload_path());
        self
    }

    /// A subdirectory named after the field, under `MEDIA_ROOT`.
    pub fn upload_to_env(mut self) -> Self {
        let f = std::sync::Arc::new(|field_name: &str| field_name.to_string());
        self.upload.upload_config.upload_to = Some(f);
        self
    }

    /// Maximum size of each file. Use `FileSize::mb(50)`, etc. Default: 10 MB.
    pub fn max_size(mut self, size: FileSize) -> Self {
        self.upload.set_max_size(size);
        self
    }

    /// Maximum number of files. Values > 1 add the `multiple` HTML attribute.
    pub fn max_files(mut self, count: usize) -> Self {
        self.upload.max_files = Some(count);
        if count > 1 {
            self.base
                .html_attributes
                .insert("multiple".to_string(), "multiple".to_string());
        }
        self
    }

    /// Marks the field as required (no file fails validation).
    pub fn required(mut self) -> Self {
        self.set_required(true, None);
        self
    }

    /// Maximum image dimensions in pixels, for [`MediaField::image`] only.
    /// A file the `image` crate cannot measure is refused: without a
    /// measurement the limit would not apply.
    pub fn max_dimensions(mut self, width: u32, height: u32) -> Self {
        self.max_width = Some(width);
        self.max_height = Some(height);
        self
    }

    /// Overrides the extension whitelist (lowercase, without dot). The content
    /// check of the kind still applies.
    pub fn allowed_extensions(mut self, exts: Vec<&str>) -> Self {
        self.upload.allowed_extensions = AllowedExtensions::new(exts);
        self
    }

    /// The image checks: signature, then dimensions when limited. `Err` holds
    /// the message to show.
    async fn check_image(&self, filename: &str) -> Result<(), String> {
        if !is_valid_image_content(filename).await {
            return Err(tf("forms.file_invalid_image", &[filename]));
        }
        // Refused when in doubt, for every format: an unreadable
        // measurement would skip the width and height limits.
        if self.max_width.is_none() && self.max_height.is_none() {
            return Ok(());
        }
        let path = filename.to_string();
        let read = tokio::task::spawn_blocking(move || read_image_dimensions(&path)).await;
        let dims = match read {
            Ok(Ok(dims)) => Some(dims),
            Ok(Err(error)) => {
                tracing::warn!(
                    field = %self.base.name,
                    file = %filename,
                    error = %error,
                    "image field: dimensions unreadable, refused"
                );
                None
            }
            Err(error) => {
                tracing::warn!(
                    field = %self.base.name,
                    error = %error,
                    "image field: dimension read task failed, refused"
                );
                None
            }
        };
        let Some((w, h)) = dims else {
            return Err(tf("forms.file_invalid_image", &[filename]));
        };
        if let Some(max_w) = self.max_width
            && w > max_w
        {
            let (w_s, mw_s) = (w.to_string(), max_w.to_string());
            return Err(tf(
                "forms.image_too_wide",
                &[filename, w_s.as_str(), mw_s.as_str()],
            ));
        }
        if let Some(max_h) = self.max_height
            && h > max_h
        {
            let (h_s, mh_s) = (h.to_string(), max_h.to_string());
            return Err(tf(
                "forms.image_too_tall",
                &[filename, h_s.as_str(), mh_s.as_str()],
            ));
        }
        Ok(())
    }
}

/// Whether `path` starts with the signature of an audio (`Audio`) or video
/// (`Video`) container. MP3 (ID3 tag or MPEG frame, which also covers ADTS
/// AAC), Ogg, WAV, FLAC, and ISO BMFF (`ftyp`: M4A, MP4, MOV) for audio;
/// ISO BMFF, WebM/Matroska (EBML) and Ogg for video.
pub(crate) async fn has_av_signature(path: &str, kind: MediaKind) -> bool {
    use tokio::io::AsyncReadExt;
    let Ok(mut f) = tokio::fs::File::open(path).await else {
        return false;
    };
    let mut buf = [0u8; 12];
    let n = f.read(&mut buf).await.unwrap_or(0);
    if n < 4 {
        return false;
    }
    let iso_bmff = n >= 8 && &buf[4..8] == b"ftyp";
    let ogg = &buf[..4] == b"OggS";
    match kind {
        MediaKind::Audio => {
            &buf[..3] == b"ID3"
                || (buf[0] == 0xFF && buf[1] & 0xE0 == 0xE0)
                || ogg
                || (n >= 12 && &buf[..4] == b"RIFF" && &buf[8..12] == b"WAVE")
                || &buf[..4] == b"fLaC"
                || iso_bmff
        }
        MediaKind::Video => iso_bmff || buf[..4] == [0x1A, 0x45, 0xDF, 0xA3] || ogg,
        MediaKind::Image => false,
    }
}

#[async_trait::async_trait]
impl FormField for MediaField {
    fn to_value(&self) -> Result<Option<sea_orm::Value>, String> {
        crate::forms::column_value::read_field(
            &self.base,
            &self.base.value,
            sea_orm::sea_query::ColumnType::String(sea_orm::sea_query::StringLen::None),
        )
    }

    fn keeps_when_empty(&self) -> bool {
        true
    }

    fn cap_max_size(&mut self, bytes: u64) {
        self.upload.cap_max_size(bytes);
    }

    fn model_max_size(&self) -> Option<u64> {
        self.upload.model_max_size
    }

    fn set_max_size_bounded(&mut self, size: FileSize) -> Result<(), String> {
        self.upload.apply_max_size_bounded(size)
    }

    fn set_submitted_value(&mut self, value: &str) {
        let staged = parse_file_list(value).iter().all(|p| is_staged_upload(p));
        if !staged {
            // Not an upload from this request: the client sent a path as text.
            tracing::warn!(field = %self.base.name, "media field: submitted value is not a staged upload, ignored");
            return;
        }
        self.set_value(value);
    }

    fn set_value(&mut self, value: &str) {
        self.upload.track_value(&mut self.base, value);
    }

    async fn validate(&mut self) -> bool {
        let files = match self.upload.check(&self.base).await {
            UploadCheck::Required(msg) => {
                self.set_error(msg);
                return false;
            }
            UploadCheck::Empty => {
                self.clear_error();
                return true;
            }
            UploadCheck::Refused(msg) => {
                self.base.value.clear();
                self.set_error(msg);
                return false;
            }
            UploadCheck::Files(files) => files,
        };

        for filename in &files {
            let refused = match self.kind {
                MediaKind::Image => self.check_image(filename).await.err(),
                MediaKind::Audio => (!has_av_signature(filename, MediaKind::Audio).await)
                    .then(|| tf("forms.file_invalid_audio", &[filename.as_str()])),
                MediaKind::Video => (!has_av_signature(filename, MediaKind::Video).await)
                    .then(|| tf("forms.file_invalid_video", &[filename.as_str()])),
            };
            if let Some(msg) = refused {
                cleanup_files(&files).await;
                self.base.value.clear();
                self.set_error(msg);
                return false;
            }
        }

        self.clear_error();
        true
    }

    async fn finalize(&mut self) -> Result<(), String> {
        self.upload.finalize(&mut self.base).await
    }

    async fn publish_uploads(&mut self) -> Result<(), String> {
        self.upload.publish().await
    }

    async fn rollback_uploads(&mut self) {
        self.upload.rollback(&mut self.base).await;
    }

    async fn complete_uploads(&mut self) {
        self.upload.complete();
    }

    fn render(&self, tera: &ATera) -> Result<String, String> {
        let mut context = self.base_context();
        context.insert("allowed_extensions", &self.upload.allowed_extensions);
        context.insert("multiple", &(self.upload.max_files.unwrap_or(1) > 1));
        context.insert("is_file", &true);
        context.insert("is_image", &(self.kind == MediaKind::Image));
        if let Some(size) = self.upload.upload_config.max_size {
            context.insert("max_size", &size);
            context.insert("max_size_mb", &(size as f64 / (1024.0 * 1024.0)));
        }
        if let Some(count) = self.upload.max_files {
            context.insert("max_files", &count);
        }
        if let Some(w) = self.max_width {
            context.insert("max_width", &w);
        }
        if let Some(h) = self.max_height {
            context.insert("max_height", &h);
        }
        tera.render(&self.base.template_name, &context)
            .map_err(|e| {
                tf(
                    "forms.finalize_error",
                    &[&self.base.template_name, &e.to_string()],
                )
                .to_string()
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::static_files::{MEDIA_ENV_LOCK, restore_media_root};
    use std::fs;

    /// A media root of its own, with a staging folder: what a request's
    /// upload leaves. Removed, and `MEDIA_ROOT` put back, when dropped.
    struct Media {
        root: PathBuf,
        staging: PathBuf,
    }

    impl Media {
        fn new() -> Self {
            let root = std::env::temp_dir().join(format!("rq_av_{}", uuid::Uuid::new_v4()));
            let staging = root.join(format!(".staging-{}", uuid::Uuid::new_v4()));
            fs::create_dir_all(&staging).unwrap();
            // SAFETY: the tests touching MEDIA_ROOT hold `MEDIA_ENV_LOCK`.
            unsafe { std::env::set_var("MEDIA_ROOT", root.to_str().unwrap()) };
            Self { root, staging }
        }

        fn staged(&self, name: &str, bytes: &[u8]) -> String {
            let path = self.staging.join(name);
            fs::write(&path, bytes).unwrap();
            path.to_string_lossy().into_owned()
        }
    }

    impl Drop for Media {
        fn drop(&mut self) {
            restore_media_root();
            let _ = fs::remove_dir_all(&self.root);
        }
    }

    const MP3_ID3: &[u8] = b"ID3\x04\x00\x00\x00\x00\x00\x00\x00\x00";
    const MP3_FRAME: &[u8] = &[0xFF, 0xFB, 0x90, 0x64, 0, 0, 0, 0, 0, 0, 0, 0];
    const OGG: &[u8] = b"OggS\x00\x02\x00\x00\x00\x00\x00\x00";
    const WAV: &[u8] = b"RIFF\x24\x00\x00\x00WAVE";
    const FLAC: &[u8] = b"fLaC\x00\x00\x00\x22\x00\x00\x00\x00";
    const MP4: &[u8] = b"\x00\x00\x00\x18ftypisom";
    const WEBM: &[u8] = &[0x1A, 0x45, 0xDF, 0xA3, 0x9F, 0x42, 0x86, 0x81, 1, 0, 0, 0];
    const PNG: &[u8] = &[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0, 0, 0, 0];
    const TEXT: &[u8] = b"just some text, not media";

    #[tokio::test]
    async fn each_audio_and_video_container_is_recognised_by_its_signature() {
        let _g = MEDIA_ENV_LOCK.lock().await;
        let media = Media::new();
        for (name, bytes) in [
            ("a.mp3", MP3_ID3),
            ("b.mp3", MP3_FRAME),
            ("c.ogg", OGG),
            ("d.wav", WAV),
            ("e.flac", FLAC),
            ("f.m4a", MP4),
        ] {
            let path = media.staged(name, bytes);
            assert!(
                has_av_signature(&path, MediaKind::Audio).await,
                "audio {name}"
            );
        }
        for (name, bytes) in [("a.mp4", MP4), ("b.webm", WEBM), ("c.ogv", OGG)] {
            let path = media.staged(name, bytes);
            assert!(
                has_av_signature(&path, MediaKind::Video).await,
                "video {name}"
            );
        }
    }

    #[tokio::test]
    async fn a_file_without_the_signature_of_its_kind_is_not_recognised() {
        let _g = MEDIA_ENV_LOCK.lock().await;
        let media = Media::new();
        for (name, bytes) in [("a.mp3", TEXT), ("b.mp3", PNG), ("c.mp3", WEBM)] {
            let path = media.staged(name, bytes);
            assert!(
                !has_av_signature(&path, MediaKind::Audio).await,
                "audio {name}"
            );
        }
        for (name, bytes) in [
            ("a.mp4", TEXT),
            ("b.mp4", PNG),
            ("c.mp4", WAV),
            ("d.mp4", MP3_ID3),
        ] {
            let path = media.staged(name, bytes);
            assert!(
                !has_av_signature(&path, MediaKind::Video).await,
                "video {name}"
            );
        }
        let short = media.staged("short.mp3", b"ID");
        assert!(
            !has_av_signature(&short, MediaKind::Audio).await,
            "too short"
        );
        assert!(!has_av_signature("/nonexistent/x.mp3", MediaKind::Audio).await);
    }

    #[tokio::test]
    async fn an_audio_field_accepts_a_real_file_and_refuses_a_renamed_one() {
        let _g = MEDIA_ENV_LOCK.lock().await;
        let media = Media::new();

        let mut field = MediaField::audio("podcast");
        field.set_value(&media.staged("ok.mp3", MP3_ID3));
        assert!(field.validate().await, "{:?}", field.error());

        let renamed = media.staged("fake.mp3", TEXT);
        let mut field = MediaField::audio("podcast");
        field.set_value(&renamed);
        assert!(!field.validate().await);
        assert_eq!(
            field.error(),
            Some(&tf("forms.file_invalid_audio", &[renamed.as_str()]))
        );
        assert_eq!(field.value(), "", "the refused value is cleared");
        assert!(
            !Path::new(&renamed).exists(),
            "the refused upload is removed"
        );
    }

    #[tokio::test]
    async fn a_video_field_accepts_a_real_file_and_refuses_audio() {
        let _g = MEDIA_ENV_LOCK.lock().await;
        let media = Media::new();

        let mut field = MediaField::video("trailer");
        field.set_value(&media.staged("ok.webm", WEBM));
        assert!(field.validate().await, "{:?}", field.error());

        let audio = media.staged("song.mp4", MP3_ID3);
        let mut field = MediaField::video("trailer");
        field.set_value(&audio);
        assert!(!field.validate().await);
        assert_eq!(
            field.error(),
            Some(&tf("forms.file_invalid_video", &[audio.as_str()]))
        );
    }

    #[tokio::test]
    async fn each_kind_keeps_to_its_own_extensions() {
        let _g = MEDIA_ENV_LOCK.lock().await;
        let media = Media::new();

        let mut field = MediaField::video("trailer");
        field.set_value(&media.staged("song.mp3", MP3_ID3));
        assert!(!field.validate().await, "an .mp3 is not a video extension");

        let mut field = MediaField::audio("podcast");
        field.set_value(&media.staged("clip.mp4", MP4));
        assert!(!field.validate().await, "an .mp4 is not an audio extension");

        let mut field = MediaField::audio("podcast");
        field.set_value(&media.staged("clip.m4a", MP4));
        assert!(field.validate().await, "an .m4a is: {:?}", field.error());
    }

    #[test]
    fn each_kind_starts_from_its_own_extension_list() {
        let exts = |f: MediaField| f.upload.allowed_extensions.extensions;
        assert!(exts(MediaField::audio("a")).contains(&"mp3".to_string()));
        assert!(!exts(MediaField::audio("a")).contains(&"mp4".to_string()));
        assert!(exts(MediaField::video("v")).contains(&"webm".to_string()));
        assert!(!exts(MediaField::video("v")).contains(&"mp3".to_string()));
        assert!(exts(MediaField::image("i")).contains(&"png".to_string()));
        assert_eq!(
            MediaField::audio("a").field_type(),
            "file",
            "cleaned up like any file field"
        );
    }
}
