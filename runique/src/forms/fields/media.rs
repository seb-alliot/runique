//! `Upload`: the upload mechanism every field taking files shares —
//! what it accepts, the staged files waiting to move, the ones moved, and the
//! value they replace. `MediaField`: images, audio and video, each checked by
//! its content.
use super::file::{
    AllowedExtensions, FileSize, FileUploadConfig, IntoUploadPath, checked_upload_relative,
    cleanup_files, ensure_safe_upload_dir, is_staged_upload, is_valid_image_content,
    parse_file_list, read_image_dimensions,
};
use crate::{
    atomic::{AtomicBuilder, SourceItems},
    config::static_files::resolve_media_root,
    forms::{config::field_config::FieldConfig, traits::form_field::FormField},
    log::TraceResult,
    translation::{t, tf},
    utils::aliases::ATera,
};

/// What the visitor reads when an upload can't be written: the detail (a
/// server path, an OS error) goes to the log only.
pub(crate) fn upload_failed(detail: String) -> String {
    tracing::warn!(error = %detail, "upload not written");
    t(runique_i18n::keys::forms::error::UPLOAD_FAILED).into_owned()
}
use std::path::{Component, Path, PathBuf};

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
pub struct Upload {
    pub allowed_extensions: AllowedExtensions,
    pub upload_config: FileUploadConfig,
    pub model_max_size: Option<u64>,
    pub max_files: Option<usize>,
    pub prev_value: Option<String>,
    /// Whether the stored file a new upload takes the place of is removed once
    /// the write is committed. Off by default: that file then stays on disk.
    pub replace: bool,
    pending_uploads: Vec<PendingUpload>,
    published_uploads: Vec<PathBuf>,
}

/// What [`Upload::check`] found in a field's value.
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

impl Upload {
    pub fn new(allowed_extensions: AllowedExtensions) -> Self {
        Self {
            allowed_extensions,
            upload_config: FileUploadConfig::default(),
            model_max_size: None,
            max_files: None,
            prev_value: None,
            replace: false,
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
            return Err(crate::translation::tf(
                runique_i18n::keys::forms::error::MAX_SIZE_ABOVE_MODEL,
                &[format!("{req_mb:.1}"), format!("{lim_mb:.1}")],
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

    /// Saving an upload, on a field that removes the file it replaces.
    pub(crate) fn awaits_replaced_file(&self) -> bool {
        self.replace && !self.pending_uploads.is_empty()
    }

    /// Remembers `stored`, the file the row held before this update, as the
    /// one the pending upload replaces. A request builds its form empty, then
    /// fills it with the upload: without this, the field never learns what
    /// the row held, and `complete` has nothing to remove.
    pub(crate) fn replaced_file(&mut self, stored: &str) {
        let stored = stored.trim();
        if self.awaits_replaced_file() && !stored.is_empty() {
            self.prev_value = Some(stored.to_string());
        }
    }

    /// Presence, count, extension and size of the files in `base`'s value.
    pub(crate) async fn check(&self, base: &FieldConfig) -> UploadCheck {
        let val = base.value.trim();

        // 1. Presence validation
        if base.is_required.choice && val.is_empty() {
            let msg =
                base.is_required.message.clone().unwrap_or_else(|| {
                    t(runique_i18n::keys::forms::error::FILE_REQUIRED).to_string()
                });
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
            return UploadCheck::Refused(tf(
                runique_i18n::keys::forms::error::FILE_MAX_COUNT,
                &[&max],
            ));
        }

        // 3. Extensions + size validation
        for filename in &files {
            if !self.allowed_extensions.is_allowed(filename) {
                let exts = self.allowed_extensions.extensions.join(", ");
                cleanup_files(&files).await;
                return UploadCheck::Refused(tf(
                    runique_i18n::keys::forms::error::FILE_EXTENSION_BLOCKED,
                    &[exts.as_str()],
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
                        return UploadCheck::Refused(
                            t(runique_i18n::keys::forms::error::FILE_UNREADABLE).to_string(),
                        );
                    }
                };
                if file_size > max_bytes {
                    let max_mb = max_bytes as f64 / (1024.0 * 1024.0);
                    cleanup_files(&files).await;
                    return UploadCheck::Refused(tf(
                        runique_i18n::keys::forms::error::FILE_TOO_LARGE,
                        &[format!("{max_mb:.1}")],
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

        // Builds the stored relative path ("filename" at the root, "rel/filename" otherwise).
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

        let mut batch = AtomicBuilder::create();
        let mut destinations = Vec::with_capacity(self.pending_uploads.len());
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

            batch = batch
                .add(
                    destination.clone(),
                    SourceItems::File(upload.staged.clone()),
                    false,
                )
                .map_err(|e| format!("publish '{}': {e}", destination.display()))?;
            destinations.push(destination);
        }

        // Staging is under MEDIA_ROOT, so each file is hard-linked: published
        // whole, and never over an existing file. One batch for the field:
        // every file, or none. The batch does blocking IO, hence the thread.
        tokio::task::spawn_blocking(move || batch.start())
            .await
            .map_err(|e| format!("publish: {e}"))?
            .map_err(|e| format!("publish: {e}"))?;
        self.published_uploads.extend(destinations);

        for upload in &self.pending_uploads {
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

    /// The write is committed: forgets the pending uploads and, for a field
    /// declared with `replace`, removes the file the new upload took the place
    /// of. Never on a field taking several files: which one a new file
    /// replaces can't be told.
    pub(crate) async fn complete(&mut self) {
        if self.replace
            && self.max_files.unwrap_or(1) <= 1
            && !self.published_uploads.is_empty()
            && let Some(previous) = self.prev_value.take()
        {
            remove_replaced(&previous).await;
        }
        self.published_uploads.clear();
        self.pending_uploads.clear();
        self.prev_value = None;
    }
}

/// Removes the stored file `previous`, a path relative to `MEDIA_ROOT` as the
/// column holds it. Best effort: the write is committed, so a file that is
/// missing, refused or impossible to remove never fails the request.
///
/// The value comes from the database, yet nothing is assumed of it: only a
/// single regular file, really under `MEDIA_ROOT` once resolved on disk and
/// outside staging, is ever removed.
async fn remove_replaced(previous: &str) {
    let files = parse_file_list(previous);
    let [file] = files.as_slice() else {
        return;
    };
    let relative = Path::new(file);
    let plain = relative.components().all(|part| match part {
        Component::Normal(name) => !name.to_string_lossy().starts_with(".staging-"),
        Component::Prefix(_) | Component::RootDir | Component::CurDir | Component::ParentDir => {
            false
        }
    });
    let (Some(parent), Some(name)) = (relative.parent(), relative.file_name()) else {
        return;
    };
    if !plain {
        return;
    }
    let Ok(root) = tokio::fs::canonicalize(resolve_media_root()).await else {
        return;
    };
    // Resolved on disk, so a symlinked folder can't lead out of MEDIA_ROOT.
    let Ok(folder) = tokio::fs::canonicalize(root.join(parent)).await else {
        return;
    };
    if !folder.starts_with(&root) {
        return;
    }
    let path = folder.join(name);
    let regular = tokio::fs::symlink_metadata(&path)
        .await
        .is_ok_and(|metadata| metadata.file_type().is_file());
    if regular {
        crate::atomic::remove(&path).trace(
            crate::log::log_level!(forms.finalize),
            runique_i18n::keys::forms::log::REMOVE_REPLACED,
        );
    }
}

/// What a [`MediaField`] takes. Each kind checks the file's content — its
/// signature — not only its extension: a file renamed `.mp4` is refused.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
pub enum MediaType {
    Image,
    Audio,
    Video,
}

/// Upload field for images, audio and video: [`MediaField::image`],
/// [`::audio`](MediaField::audio), [`::video`](MediaField::video). Shares the
/// upload mechanism of every file field ([`Upload`]); on top, checks the
/// content of each file — an image's signature and dimensions, an audio or
/// video file's signature. Documents and other files: [`FileField`](super::FileField).
#[derive(Clone, Debug)]
pub struct MediaField {
    pub base: FieldConfig,
    pub media_type: MediaType,
    pub upload: Upload,
    pub max_width: Option<u32>,
    pub max_height: Option<u32>,
}

field_base!(MediaField, label, required);

impl MediaField {
    fn create(name: &str, kind: MediaType) -> Self {
        let extensions = match kind {
            MediaType::Image => AllowedExtensions::images(),
            MediaType::Audio => AllowedExtensions::audio(),
            MediaType::Video => AllowedExtensions::video(),
        };
        Self {
            base: FieldConfig::new(name, "file", "base_file.html"),
            media_type: kind,
            upload: Upload::new(extensions),
            max_width: None,
            max_height: None,
        }
    }

    /// Image: jpg, jpeg, png, gif, webp, avif. Checks the signature, and the
    /// dimensions with [`max_dimensions`](Self::max_dimensions). SVG refused.
    pub fn image(name: &str) -> Self {
        Self::create(name, MediaType::Image)
    }

    /// Audio: mp3, ogg, oga, wav, m4a, flac, aac. Checks the signature.
    pub fn audio(name: &str) -> Self {
        Self::create(name, MediaType::Audio)
    }

    /// Video: mp4, webm, mov, ogv. Checks the signature.
    pub fn video(name: &str) -> Self {
        Self::create(name, MediaType::Video)
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

    /// Once a new upload is saved, removes the stored file it takes the place
    /// of. Without it that file stays on disk, and removing it is your call.
    /// No effect on a field taking several files (`max_files` above 1).
    pub fn replace(mut self) -> Self {
        self.upload.replace = true;
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
            return Err(t(runique_i18n::keys::forms::error::FILE_INVALID_IMAGE).into_owned());
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
            return Err(t(runique_i18n::keys::forms::error::FILE_INVALID_IMAGE).into_owned());
        };
        if let Some(max_w) = self.max_width
            && w > max_w
        {
            return Err(tf(
                runique_i18n::keys::forms::error::IMAGE_TOO_WIDE,
                &[max_w],
            ));
        }
        if let Some(max_h) = self.max_height
            && h > max_h
        {
            return Err(tf(
                runique_i18n::keys::forms::error::IMAGE_TOO_TALL,
                &[max_h],
            ));
        }
        Ok(())
    }
}

/// Whether `path` starts with the signature of an audio (`Audio`) or video
/// (`Video`) container. MP3 (ID3 tag or MPEG frame, which also covers ADTS
/// AAC), Ogg, WAV, FLAC, and ISO BMFF (`ftyp`: M4A, MP4, MOV) for audio;
/// ISO BMFF, WebM/Matroska (EBML) and Ogg for video.
pub(crate) async fn has_av_signature(path: &str, kind: MediaType) -> bool {
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
        MediaType::Audio => {
            &buf[..3] == b"ID3"
                || (buf[0] == 0xFF && buf[1] & 0xE0 == 0xE0)
                || ogg
                || (n >= 12 && &buf[..4] == b"RIFF" && &buf[8..12] == b"WAVE")
                || &buf[..4] == b"fLaC"
                || iso_bmff
        }
        MediaType::Video => iso_bmff || buf[..4] == [0x1A, 0x45, 0xDF, 0xA3] || ogg,
        MediaType::Image => false,
    }
}

#[async_trait::async_trait]
impl FormField for MediaField {
    fn to_value(&self) -> Result<Option<sea_orm::Value>, String> {
        crate::forms::data::column_value::read_field(
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
            let refused = match self.media_type {
                MediaType::Image => self.check_image(filename).await.err(),
                MediaType::Audio => (!has_av_signature(filename, MediaType::Audio).await)
                    .then(|| t(runique_i18n::keys::forms::error::FILE_INVALID_AUDIO).into_owned()),
                MediaType::Video => (!has_av_signature(filename, MediaType::Video).await)
                    .then(|| t(runique_i18n::keys::forms::error::FILE_INVALID_VIDEO).into_owned()),
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
        self.upload
            .finalize(&mut self.base)
            .await
            .map_err(upload_failed)
    }

    fn awaits_replaced_file(&self) -> bool {
        self.upload.awaits_replaced_file()
    }

    fn replaced_file(&mut self, stored: &str) {
        self.upload.replaced_file(stored);
    }

    async fn publish_uploads(&mut self) -> Result<(), String> {
        self.upload.publish().await.map_err(upload_failed)
    }

    async fn rollback_uploads(&mut self) {
        self.upload.rollback(&mut self.base).await;
    }

    async fn complete_uploads(&mut self) {
        self.upload.complete().await;
    }

    fn render(&self, tera: &ATera) -> Result<String, String> {
        let mut context = self.base_context();
        context.insert("allowed_extensions", &self.upload.allowed_extensions);
        context.insert("multiple", &(self.upload.max_files.unwrap_or(1) > 1));
        context.insert("is_file", &true);
        context.insert("is_image", &(self.media_type == MediaType::Image));
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
        self.render_template(tera, &context)
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
                has_av_signature(&path, MediaType::Audio).await,
                "audio {name}"
            );
        }
        for (name, bytes) in [("a.mp4", MP4), ("b.webm", WEBM), ("c.ogv", OGG)] {
            let path = media.staged(name, bytes);
            assert!(
                has_av_signature(&path, MediaType::Video).await,
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
                !has_av_signature(&path, MediaType::Audio).await,
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
                !has_av_signature(&path, MediaType::Video).await,
                "video {name}"
            );
        }
        let short = media.staged("short.mp3", b"ID");
        assert!(
            !has_av_signature(&short, MediaType::Audio).await,
            "too short"
        );
        assert!(!has_av_signature("/nonexistent/x.mp3", MediaType::Audio).await);
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
            Some(&t(runique_i18n::keys::forms::error::FILE_INVALID_AUDIO).into_owned()),
            "the reason only: no name, no staging path"
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
            Some(&t(runique_i18n::keys::forms::error::FILE_INVALID_VIDEO).into_owned()),
            "the reason only: no name, no staging path"
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

    // Written from cargo-mutants survivors: `MediaField` delegates its upload to
    // `Upload`, and these delegations were only tested through `FileField`.

    /// `set_value` then `validate`, as a real submission does, then `finalize`.
    async fn finalized(field: &mut MediaField, staged: &str) {
        field.set_value(staged);
        assert!(field.validate().await, "{:?}", field.error());
        field.finalize().await.unwrap();
    }

    #[tokio::test]
    async fn publishing_moves_the_file_of_a_media_field_out_of_staging() {
        let _g = MEDIA_ENV_LOCK.lock().await;
        let media = Media::new();
        let staged = media.staged("ep.mp3", MP3_ID3);
        let mut field = MediaField::audio("podcast");

        finalized(&mut field, &staged).await;
        assert_eq!(field.value(), "ep.mp3", "finalize stores the relative path");
        assert!(Path::new(&staged).exists(), "not moved before the write");
        assert!(!media.root.join("ep.mp3").exists());

        field.publish_uploads().await.unwrap();
        assert!(media.root.join("ep.mp3").exists());
        assert!(!Path::new(&staged).exists());
    }

    #[tokio::test]
    async fn rolling_back_takes_the_file_of_a_media_field_back_and_restores_the_value() {
        let _g = MEDIA_ENV_LOCK.lock().await;
        let media = Media::new();
        fs::write(media.root.join("old.mp3"), MP3_ID3).unwrap();
        let mut field = MediaField::audio("podcast");
        field.set_value("old.mp3");

        finalized(&mut field, &media.staged("new.mp3", MP3_ID3)).await;
        field.publish_uploads().await.unwrap();
        assert!(media.root.join("new.mp3").exists());

        field.rollback_uploads().await;
        assert!(!media.root.join("new.mp3").exists(), "taken back");
        assert!(media.root.join("old.mp3").exists(), "the old file stays");
        assert_eq!(field.value(), "old.mp3");
    }

    #[tokio::test]
    async fn a_completed_upload_of_a_media_field_is_no_longer_taken_back() {
        let _g = MEDIA_ENV_LOCK.lock().await;
        let media = Media::new();
        let mut field = MediaField::audio("podcast");

        finalized(&mut field, &media.staged("kept.mp3", MP3_ID3)).await;
        field.publish_uploads().await.unwrap();
        field.complete_uploads().await;

        field.rollback_uploads().await;
        assert!(media.root.join("kept.mp3").exists());
        assert_eq!(field.value(), "kept.mp3");
    }

    #[tokio::test]
    async fn a_media_field_only_takes_a_submitted_value_that_is_a_staged_upload() {
        let _g = MEDIA_ENV_LOCK.lock().await;
        let media = Media::new();
        let staged = media.staged("ep.mp3", MP3_ID3);

        let mut field = MediaField::audio("podcast");
        field.set_submitted_value(&staged);
        assert_eq!(field.value(), staged);

        let mut field = MediaField::audio("podcast");
        field.set_submitted_value("/etc/passwd");
        assert_eq!(field.value(), "", "a path sent as text is ignored");
    }

    #[test]
    fn a_media_field_gives_its_path_as_its_column_value() {
        let mut field = MediaField::audio("podcast");
        assert_eq!(field.to_value(), Ok(None), "empty");
        field.base.value = "ep.mp3".to_string();
        assert_eq!(field.to_value(), Ok(Some(sea_orm::Value::from("ep.mp3"))));
    }

    #[test]
    fn a_media_field_applies_the_size_cap_it_is_given() {
        let mut field = MediaField::video("clip");
        FormField::cap_max_size(&mut field, 100);
        assert_eq!(field.upload.upload_config.max_size, Some(100));
        FormField::cap_max_size(&mut field, 200);
        assert_eq!(
            field.upload.upload_config.max_size,
            Some(100),
            "a cap never raises the limit"
        );
    }

    #[test]
    fn only_more_than_one_file_makes_the_input_multiple() {
        for (count, multiple) in [(0, false), (1, false), (2, true)] {
            let field = MediaField::image("photo").max_files(count);
            assert_eq!(
                field.base.html_attributes.contains_key("multiple"),
                multiple,
                "{count}"
            );
        }
    }

    #[test]
    fn the_template_learns_whether_several_files_are_allowed() {
        let mut tera = tera::Tera::default();
        tera.add_raw_template("base_file.html", "{{ multiple }}")
            .unwrap();
        let tera = std::sync::Arc::new(tera);
        for (field, multiple) in [
            (MediaField::image("photo"), "false"),
            (MediaField::image("photo").max_files(1), "false"),
            (MediaField::image("photo").max_files(2), "true"),
        ] {
            assert_eq!(field.render(&tera).unwrap(), multiple);
        }
    }

    /// One limit alone is enough to measure the image: a `max_width` without
    /// `max_height`, or the reverse, must not skip the check.
    #[tokio::test]
    async fn a_single_dimension_limit_is_enforced() {
        let _g = MEDIA_ENV_LOCK.lock().await;
        let media = Media::new();
        let photo = |name: &str| {
            let path = media.staging.join(name);
            image::RgbImage::new(8, 2).save(&path).unwrap();
            path.to_string_lossy().into_owned()
        };
        let checked = async |name: &str, width: Option<u32>, height: Option<u32>| {
            let mut field = MediaField::image("photo");
            field.max_width = width;
            field.max_height = height;
            field.set_value(&photo(name));
            field.validate().await
        };

        assert!(!checked("wide.png", Some(4), None).await, "too wide");
        assert!(!checked("tall.png", None, Some(1)).await, "too tall");
        assert!(
            checked("w_ok.png", Some(8), None).await,
            "width at the limit"
        );
        assert!(
            checked("h_ok.png", None, Some(2)).await,
            "height at the limit"
        );
    }

    #[tokio::test]
    async fn a_signature_needs_every_one_of_its_bytes() {
        const FRAME_LOW: &[u8] = &[0xFF, 0x00, 0, 0];
        const FRAME_HIGH: &[u8] = &[0x00, 0xE0, 0, 0];
        let _g = MEDIA_ENV_LOCK.lock().await;
        let media = Media::new();
        let cases: [(&str, &[u8], bool); 6] = [
            ("three.mp3", b"ID3", false),
            ("four.ogg", b"OggS", true),
            ("frame_low.mp3", FRAME_LOW, false),
            ("frame_high.mp3", FRAME_HIGH, false),
            ("riff_avi.wav", b"RIFF\x24\x00\x00\x00AVI ", false),
            ("no_riff.wav", b"JUNK\x24\x00\x00\x00WAVE", false),
        ];
        for (name, bytes, accepted) in cases {
            let path = media.staged(name, bytes);
            assert_eq!(
                has_av_signature(&path, MediaType::Audio).await,
                accepted,
                "{name}"
            );
        }
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

    // ── `.replace()` ─────────────────────────────────────────────────────────

    /// `field` holding `old.mp3`, then a new upload saved up to its publication.
    async fn replaced_by_new(media: &Media, field: &mut MediaField) {
        fs::write(media.root.join("old.mp3"), MP3_ID3).unwrap();
        field.set_value("old.mp3");
        finalized(field, &media.staged("new.mp3", MP3_ID3)).await;
        field.publish_uploads().await.unwrap();
    }

    #[tokio::test]
    async fn a_field_that_replaces_removes_the_old_file_once_the_write_is_committed() {
        let _g = MEDIA_ENV_LOCK.lock().await;
        let media = Media::new();
        let mut field = MediaField::audio("podcast").replace();

        replaced_by_new(&media, &mut field).await;
        assert!(media.root.join("old.mp3").exists(), "kept until the commit");

        field.complete_uploads().await;
        assert!(media.root.join("new.mp3").exists());
        assert!(!media.root.join("old.mp3").exists());
        assert_eq!(field.value(), "new.mp3");
    }

    // ── `.replace()` in a request ────────────────────────────────────────────
    // A request builds its form empty and fills it with the upload: the field
    // never held the stored value. The update tells it (`replaced_file`).

    #[tokio::test]
    async fn an_update_tells_a_replacing_field_which_file_to_remove() {
        let _g = MEDIA_ENV_LOCK.lock().await;
        let media = Media::new();
        fs::write(media.root.join("old.mp3"), MP3_ID3).unwrap();
        let mut field = MediaField::audio("podcast").replace();
        assert!(!field.awaits_replaced_file(), "no upload yet");

        finalized(&mut field, &media.staged("new.mp3", MP3_ID3)).await;
        assert!(field.awaits_replaced_file());
        field.replaced_file("old.mp3");
        field.publish_uploads().await.unwrap();
        assert!(media.root.join("old.mp3").exists(), "kept until the commit");

        field.complete_uploads().await;
        assert!(media.root.join("new.mp3").exists());
        assert!(!media.root.join("old.mp3").exists());
    }

    #[tokio::test]
    async fn an_update_removes_nothing_for_a_field_that_does_not_replace() {
        let _g = MEDIA_ENV_LOCK.lock().await;
        let media = Media::new();
        fs::write(media.root.join("old.mp3"), MP3_ID3).unwrap();
        let mut field = MediaField::audio("podcast");

        finalized(&mut field, &media.staged("new.mp3", MP3_ID3)).await;
        assert!(!field.awaits_replaced_file());
        field.replaced_file("old.mp3");
        field.publish_uploads().await.unwrap();
        field.complete_uploads().await;

        assert!(media.root.join("new.mp3").exists());
        assert!(media.root.join("old.mp3").exists());
    }

    // Told what the row holds, but no new file was sent: nothing is replaced.
    #[tokio::test]
    async fn an_update_without_a_new_upload_removes_nothing() {
        let _g = MEDIA_ENV_LOCK.lock().await;
        let media = Media::new();
        fs::write(media.root.join("old.mp3"), MP3_ID3).unwrap();
        let mut field = MediaField::audio("podcast").replace();

        field.replaced_file("old.mp3");
        assert_eq!(field.upload.prev_value, None);
        field.publish_uploads().await.unwrap();
        field.complete_uploads().await;

        assert!(media.root.join("old.mp3").exists());
    }

    #[tokio::test]
    async fn a_field_that_does_not_replace_leaves_the_old_file() {
        let _g = MEDIA_ENV_LOCK.lock().await;
        let media = Media::new();
        let mut field = MediaField::audio("podcast");

        replaced_by_new(&media, &mut field).await;
        field.complete_uploads().await;

        assert!(media.root.join("new.mp3").exists());
        assert!(media.root.join("old.mp3").exists());
    }

    #[tokio::test]
    async fn a_field_that_replaces_keeps_the_old_file_when_the_write_is_rolled_back() {
        let _g = MEDIA_ENV_LOCK.lock().await;
        let media = Media::new();
        let mut field = MediaField::audio("podcast").replace();

        replaced_by_new(&media, &mut field).await;
        field.rollback_uploads().await;

        assert!(!media.root.join("new.mp3").exists());
        assert!(media.root.join("old.mp3").exists());
        assert_eq!(field.value(), "old.mp3");
    }

    // Nothing stored before, or a stored file already gone from the disk: the
    // new one is saved all the same.
    #[tokio::test]
    async fn replacing_with_nothing_to_remove_is_not_an_error() {
        let _g = MEDIA_ENV_LOCK.lock().await;
        let media = Media::new();

        let mut first = MediaField::audio("podcast").replace();
        finalized(&mut first, &media.staged("first.mp3", MP3_ID3)).await;
        first.publish_uploads().await.unwrap();
        first.complete_uploads().await;
        assert!(media.root.join("first.mp3").exists());

        let mut gone = MediaField::audio("podcast").replace();
        gone.set_value("vanished.mp3");
        finalized(&mut gone, &media.staged("second.mp3", MP3_ID3)).await;
        gone.publish_uploads().await.unwrap();
        gone.complete_uploads().await;
        assert!(media.root.join("second.mp3").exists());
    }

    // The form is saved again without a new file: nothing was replaced.
    #[tokio::test]
    async fn a_field_saved_without_a_new_upload_keeps_its_file() {
        let _g = MEDIA_ENV_LOCK.lock().await;
        let media = Media::new();
        fs::write(media.root.join("old.mp3"), MP3_ID3).unwrap();
        let mut field = MediaField::audio("podcast").replace();
        field.set_value("old.mp3");

        field.publish_uploads().await.unwrap();
        field.complete_uploads().await;

        assert!(media.root.join("old.mp3").exists());
    }

    #[tokio::test]
    async fn a_field_taking_several_files_never_removes_one() {
        let _g = MEDIA_ENV_LOCK.lock().await;
        let media = Media::new();
        let mut field = MediaField::audio("podcast").max_files(3).replace();

        replaced_by_new(&media, &mut field).await;
        field.complete_uploads().await;

        assert!(media.root.join("new.mp3").exists());
        assert!(media.root.join("old.mp3").exists());
    }

    // The value a column holds is not trusted: a path leaving MEDIA_ROOT, an
    // absolute one, a staged one or several at once remove nothing.
    #[tokio::test]
    async fn a_previous_value_outside_the_media_root_removes_nothing() {
        let _g = MEDIA_ENV_LOCK.lock().await;
        let media = Media::new();
        let outside = media
            .root
            .parent()
            .unwrap()
            .join(format!("rq_victim_{}.mp3", uuid::Uuid::new_v4()));
        fs::write(&outside, MP3_ID3).unwrap();
        let name = outside.file_name().unwrap().to_str().unwrap().to_string();
        fs::create_dir(media.root.join(".staging-other")).unwrap();
        fs::write(media.root.join(".staging-other/theirs.mp3"), MP3_ID3).unwrap();
        fs::write(media.root.join("b.mp3"), MP3_ID3).unwrap();

        for (index, previous) in [
            format!("../{name}"),
            outside.to_string_lossy().into_owned(),
            ".staging-other/theirs.mp3".to_string(),
            "old.mp3,b.mp3".to_string(),
            "./old.mp3".to_string(),
        ]
        .into_iter()
        .enumerate()
        {
            let mut field = MediaField::audio("podcast").replace();
            fs::write(media.root.join("old.mp3"), MP3_ID3).unwrap();
            field.set_value("old.mp3");
            let staged = media.staged(&format!("new{index}.mp3"), MP3_ID3);
            finalized(&mut field, &staged).await;
            field.publish_uploads().await.unwrap();
            field.upload.prev_value = Some(previous.clone());

            field.complete_uploads().await;

            assert!(outside.exists(), "{previous}");
            assert!(
                media.root.join(".staging-other/theirs.mp3").exists(),
                "{previous}"
            );
            assert!(media.root.join("old.mp3").exists(), "{previous}");
            assert!(media.root.join("b.mp3").exists(), "{previous}");
        }
        let _ = fs::remove_file(&outside);
    }

    // A symlink stored as the previous file is left alone, and so is its target.
    #[cfg(unix)]
    #[tokio::test]
    async fn a_previous_value_that_is_a_symlink_removes_nothing() {
        let _g = MEDIA_ENV_LOCK.lock().await;
        let media = Media::new();
        fs::write(media.root.join("target.mp3"), MP3_ID3).unwrap();
        std::os::unix::fs::symlink(media.root.join("target.mp3"), media.root.join("link.mp3"))
            .unwrap();
        let mut field = MediaField::audio("podcast").replace();

        replaced_by_new(&media, &mut field).await;
        field.upload.prev_value = Some("link.mp3".into());
        field.complete_uploads().await;

        assert!(media.root.join("target.mp3").exists());
        assert!(media.root.join("link.mp3").exists());
    }

    // ── Publication through the batch ────────────────────────────────────────

    #[tokio::test]
    async fn publishing_leaves_none_of_the_batch_s_own_files_in_the_media_root() {
        let _g = MEDIA_ENV_LOCK.lock().await;
        let media = Media::new();
        let mut field = MediaField::audio("podcast");

        finalized(&mut field, &media.staged("ep.mp3", MP3_ID3)).await;
        field.publish_uploads().await.unwrap();

        let leftovers: Vec<_> = fs::read_dir(&media.root)
            .unwrap()
            .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
            .filter(|name| name.contains(".runique-"))
            .collect();
        assert!(leftovers.is_empty(), "{leftovers:?}");
    }

    // The name of the second file is already taken: the first one is not
    // published either, and both are still in staging for the request to remove.
    #[tokio::test]
    async fn the_files_of_one_field_are_published_all_or_none() {
        let _g = MEDIA_ENV_LOCK.lock().await;
        let media = Media::new();
        let first = media.staged("a.mp3", MP3_ID3);
        let second = media.staged("b.mp3", MP3_ID3);
        let mut field = MediaField::audio("podcast").max_files(2);
        finalized(&mut field, &format!("{first},{second}")).await;
        fs::write(media.root.join("b.mp3"), TEXT).unwrap();

        assert!(field.publish_uploads().await.is_err());

        assert!(!media.root.join("a.mp3").exists());
        assert_eq!(fs::read(media.root.join("b.mp3")).unwrap(), TEXT);
        assert!(Path::new(&first).exists());
        assert!(Path::new(&second).exists());
    }

    // The file is already published under that name: publishing again is
    // refused, and the published file is the first one.
    #[tokio::test]
    async fn a_published_name_is_never_overwritten() {
        let _g = MEDIA_ENV_LOCK.lock().await;
        let media = Media::new();
        fs::write(media.root.join("ep.mp3"), TEXT).unwrap();
        let mut field = MediaField::audio("podcast");
        finalized(&mut field, &media.staged("ep.mp3", MP3_ID3)).await;

        assert!(field.publish_uploads().await.is_err());

        assert_eq!(fs::read(media.root.join("ep.mp3")).unwrap(), TEXT);
    }

    #[tokio::test]
    async fn a_media_field_replaces_inside_its_upload_folder() {
        let _g = MEDIA_ENV_LOCK.lock().await;
        let media = Media::new();
        fs::create_dir(media.root.join("shows")).unwrap();
        fs::write(media.root.join("shows/old.mp3"), MP3_ID3).unwrap();
        fs::write(media.root.join("old.mp3"), MP3_ID3).unwrap();
        let mut field = MediaField::audio("podcast").upload_to("shows").replace();
        field.set_value("shows/old.mp3");

        finalized(&mut field, &media.staged("new.mp3", MP3_ID3)).await;
        field.publish_uploads().await.unwrap();
        field.complete_uploads().await;

        assert_eq!(field.value(), "shows/new.mp3");
        assert!(media.root.join("shows/new.mp3").exists());
        assert!(!media.root.join("shows/old.mp3").exists());
        assert!(
            media.root.join("old.mp3").exists(),
            "same name, other folder"
        );
    }
}
