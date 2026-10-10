//! File Field: `FileField` with type, size, and upload path validation.
use crate::config::static_files::resolve_media_root;
use crate::utils::aliases::ATera;
use crate::utils::trad::{t, tf};
use crate::{
    config::StaticConfig,
    forms::base::{CommonFieldConfig, FieldConfig, FormField},
};

/// Accepts an upload path in several forms:
/// - `"media/avatars"` (&str)
/// - `String`
/// - `&StaticConfig` (uses `media_root` from .env)
pub trait IntoUploadPath {
    fn into_upload_path(self) -> String;
}

impl IntoUploadPath for &str {
    fn into_upload_path(self) -> String {
        self.to_string()
    }
}

impl IntoUploadPath for String {
    fn into_upload_path(self) -> String {
        self
    }
}

/// The root of `MEDIA_ROOT` itself: `finalize` already prefixes every upload
/// path with it, so passing it again would nest it into itself.
impl IntoUploadPath for &StaticConfig {
    fn into_upload_path(self) -> String {
        String::new()
    }
}
use async_trait::async_trait;
use image::ImageReader;
use serde::Serialize;
use std::path::Path;
use std::sync::Arc;
use tokio::io::AsyncReadExt;

/// Deletes uploaded files from disk (cleanup on validation failure)
/// Whether `path` is a file `parse_multipart` staged for this kind of request:
/// directly inside `{MEDIA_ROOT}/.staging-<uuid>/`. Resolved on disk, so `..`
/// or a symlink can't make another file pass for one.
///
/// A file field's value can also arrive as plain text (urlencoded body, or a
/// multipart part without a filename): only a staged path may ever be read,
/// moved or deleted by the field.
pub(crate) fn is_staged_upload(path: &str) -> bool {
    let (Ok(file), Ok(root)) = (
        std::fs::canonicalize(path),
        std::fs::canonicalize(resolve_media_root()),
    ) else {
        return false;
    };
    let Some(dir) = file.parent() else {
        return false;
    };
    file.is_file()
        && dir.parent() == Some(root.as_path())
        && dir
            .file_name()
            .and_then(|n| n.to_str())
            .is_some_and(|n| n.starts_with(".staging-"))
}

async fn cleanup_files(files: &[String]) {
    for path in files.iter().filter(|p| is_staged_upload(p)) {
        if let Err(e) = tokio::fs::remove_file(path).await
            && e.kind() != std::io::ErrorKind::NotFound
        {
            tracing::warn!(path = %path, error = %e, "cleanup_files: remove failed");
        }
    }
}

/// Checks if the file is a valid image using magic bytes.
/// Covers JPEG, PNG, GIF, WebP, and ISO BMFF containers (AVIF, HEIC, HEIF).
async fn is_valid_image_content(path: &str) -> bool {
    if path.to_lowercase().ends_with(".svg") {
        return false;
    }
    let Ok(mut f) = tokio::fs::File::open(path).await else {
        return false;
    };
    let mut buf = [0u8; 12];
    let n = f.read(&mut buf).await.unwrap_or(0);
    if n < 4 {
        return false;
    }
    // JPEG
    if buf[..3] == [0xFF, 0xD8, 0xFF] {
        return true;
    }
    // PNG
    if buf[..4] == [0x89, 0x50, 0x4E, 0x47] {
        return true;
    }
    // GIF
    if buf[..4] == [0x47, 0x49, 0x46, 0x38] {
        return true;
    }
    // WebP (RIFF....WEBP)
    if n >= 12 && &buf[..4] == b"RIFF" && &buf[8..12] == b"WEBP" {
        return true;
    }
    // AVIF / HEIC / HEIF — ISO Base Media File Format: ftyp box at offset 4
    if n >= 8 && &buf[4..8] == b"ftyp" {
        return true;
    }
    false
}

/// Moves a staged file to its destination. `rename` fails with `CrossesDevices`
/// when the staging directory and `MEDIA_ROOT` are on different filesystems
/// (e.g. `/tmp` on tmpfs vs. a mounted media volume) — falls back to copy+remove.
async fn move_file(src: &Path, dest: &Path) -> std::io::Result<()> {
    match tokio::fs::rename(src, dest).await {
        Ok(()) => Ok(()),
        Err(e) if e.kind() == std::io::ErrorKind::CrossesDevices => {
            tracing::debug!(src = %src.display(), dest = %dest.display(), "cross-device move, falling back to copy");
            tokio::fs::copy(src, dest).await?;
            if let Err(e) = tokio::fs::remove_file(src).await {
                tracing::warn!(path = %src.display(), error = %e, "cross-device move: leftover staged file removal failed");
            }
            Ok(())
        }
        Err(e) => Err(e),
    }
}

/// Parses the field value into a list of file paths
fn parse_file_list(val: &str) -> Vec<&str> {
    val.split(',')
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .collect()
}

/// Determines which validation rules and allowed extensions apply to a [`FileField`].
#[derive(Debug, Clone, Serialize)]
pub enum FileFieldType {
    None,
    Image,
    Document,
    Any,
}

/// Extension whitelist applied during file validation.
/// SVG is always rejected for images regardless of the list (XSS risk).
#[derive(Debug, Clone, Serialize)]
pub struct AllowedExtensions {
    pub extensions: Vec<String>,
}

impl AllowedExtensions {
    /// Custom extension list (lowercase, without dot: `vec!["jpg", "png"]`).
    pub fn new(extensions: Vec<&str>) -> Self {
        Self {
            extensions: extensions.iter().map(|s| s.to_string()).collect(),
        }
    }

    /// Preset for web images: jpg, jpeg, png, gif, webp, avif.
    pub fn images() -> Self {
        Self::new(vec!["jpg", "jpeg", "png", "gif", "webp", "avif"])
    }

    /// Preset for documents: pdf, doc, docx, txt, odt.
    pub fn documents() -> Self {
        Self::new(vec!["pdf", "doc", "docx", "txt", "odt"])
    }

    /// No extension filter (any extension accepted, SVG still rejected).
    pub fn any() -> Self {
        Self { extensions: vec![] }
    }

    /// Returns whether `filename`'s extension passes this whitelist.
    /// Always rejects `.svg` regardless of the list; an empty list allows
    /// any other extension.
    pub fn is_allowed(&self, filename: &str) -> bool {
        if filename.to_lowercase().ends_with(".svg") {
            return false;
        }
        if self.extensions.is_empty() {
            return true;
        }

        Path::new(filename)
            .extension()
            .and_then(|ext| ext.to_str())
            .map(|ext_str| {
                let ext_lower = ext_str.to_lowercase();
                self.extensions
                    .iter()
                    .any(|allowed| allowed.to_lowercase() == ext_lower)
            })
            .unwrap_or(false)
    }
}

pub type UploadPathFn = Option<Arc<dyn Fn(&str) -> String + Send + Sync>>;

/// Explicit file size unit — avoids ambiguity between bytes and MB.
///
/// Usage: `FileSize::mb(5)`, `FileSize::kb(512)`, `FileSize::gb(1)`
#[derive(Debug, Clone, Copy)]
pub struct FileSize(u64);

impl FileSize {
    /// Creates a size expressed in raw bytes.
    pub fn bytes(n: u64) -> Self {
        Self(n)
    }
    /// Creates a size expressed in kilobytes (`n * 1024` bytes).
    pub fn kb(n: u64) -> Self {
        Self(n * 1024)
    }
    /// Creates a size expressed in megabytes (`n * 1024 * 1024` bytes).
    pub fn mb(n: u64) -> Self {
        Self(n * 1024 * 1024)
    }
    /// Creates a size expressed in gigabytes (`n * 1024 * 1024 * 1024` bytes).
    pub fn gb(n: u64) -> Self {
        Self(n * 1024 * 1024 * 1024)
    }

    /// Returns the size in bytes.
    pub fn as_bytes(self) -> u64 {
        self.0
    }
}

impl From<FileSize> for u64 {
    fn from(s: FileSize) -> u64 {
        s.0
    }
}

/// Default limit if `.max_size()` is never called (10 MB default).
const DEFAULT_MAX_SIZE: u64 = 10 * 1024 * 1024;

/// File upload configuration
#[derive(Clone, Serialize)]
pub struct FileUploadConfig {
    #[serde(skip_serializing)]
    pub upload_to: UploadPathFn,
    pub max_size: Option<u64>,
}

impl Default for FileUploadConfig {
    fn default() -> Self {
        Self {
            upload_to: None,
            max_size: Some(DEFAULT_MAX_SIZE),
        }
    }
}

impl std::fmt::Debug for FileUploadConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FileUploadConfig")
            .field("upload_to", &self.upload_to.as_ref().map(|_| "Fn(...)"))
            .field("max_size", &self.max_size)
            .finish()
    }
}

impl FileUploadConfig {
    /// Creates a config with no upload path set and the default 10 MB max size.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets a fixed upload destination path, overriding the field-name-derived default.
    pub fn upload_to(mut self, path: String) -> Self {
        let f = Arc::new(move |_field_name: &str| path.clone());
        self.upload_to = Some(f);
        self
    }

    /// Sets the maximum accepted upload size.
    pub fn max_size(mut self, size: FileSize) -> Self {
        self.max_size = Some(size.as_bytes());
        self
    }
}

/// File upload field. Construct with [`FileField::image`], [`::document`](FileField::document),
/// or [`::any`](FileField::any). Validates extension, size, and for images, magic bytes + dimensions.
#[derive(Clone, Debug)]
pub struct FileField {
    pub base: FieldConfig,
    pub field_type: FileFieldType,
    pub allowed_extensions: AllowedExtensions,
    pub upload_config: FileUploadConfig,
    pub model_max_size: Option<u64>,
    pub max_files: Option<usize>,
    pub max_width: Option<u32>,
    pub max_height: Option<u32>,
    pub prev_value: Option<String>,
}

impl CommonFieldConfig for FileField {
    fn get_field_config(&self) -> &FieldConfig {
        &self.base
    }

    fn get_field_config_mut(&mut self) -> &mut FieldConfig {
        &mut self.base
    }
}

impl FileField {
    /// Low-level constructor. Prefer [`image`](FileField::image), [`document`](FileField::document), or [`any`](FileField::any).
    pub fn create(name: &str, type_field: &str, format: FileFieldType) -> Self {
        let extensions = match format {
            FileFieldType::Image => AllowedExtensions::images(),
            FileFieldType::Document => AllowedExtensions::documents(),
            FileFieldType::Any => AllowedExtensions::any(),
            FileFieldType::None => AllowedExtensions::new(vec![]),
        };

        Self {
            base: FieldConfig::new(name, type_field, "base_file.html"),
            field_type: format,
            allowed_extensions: extensions,
            upload_config: FileUploadConfig::default(),
            model_max_size: None,
            max_files: None,
            max_width: None,
            max_height: None,
            prev_value: None,
        }
    }

    /// Image upload field. Validates magic bytes (JPEG/PNG/GIF/WebP/AVIF). SVG rejected.
    pub fn image(name: &str) -> Self {
        Self::create(name, "file", FileFieldType::Image)
    }

    /// Document upload field. Accepts pdf, doc, docx, txt, odt.
    pub fn document(name: &str) -> Self {
        Self::create(name, "file", FileFieldType::Document)
    }

    /// Generic upload field. No extension restriction (SVG still rejected).
    pub fn any(name: &str) -> Self {
        Self::create(name, "file", FileFieldType::Any)
    }

    /// Overrides the auto-generated label.
    pub fn label(mut self, label: &str) -> Self {
        self.base.label = label.to_string();
        self
    }

    /// Destination directory, relative to `MEDIA_ROOT` (`"avatars"` →
    /// `{MEDIA_ROOT}/avatars/`). Accepts `&str`, `String`, or `&StaticConfig`
    /// for the root of `MEDIA_ROOT`.
    pub fn upload_to(mut self, path: impl IntoUploadPath) -> Self {
        self.upload_config = self.upload_config.upload_to(path.into_upload_path());
        self
    }

    /// A subdirectory named after the field, under `MEDIA_ROOT`
    /// (`"avatar"` → `{MEDIA_ROOT}/avatar/`).
    pub fn upload_to_env(mut self) -> Self {
        let f = Arc::new(|field_name: &str| field_name.to_string());
        self.upload_config.upload_to = Some(f);
        self
    }

    /// Maximum upload size. Use `FileSize::mb(5)`, `::kb(512)`, etc. Default: 10 MB.
    pub fn max_size(mut self, size: FileSize) -> Self {
        let bytes = size.as_bytes();
        self.model_max_size = Some(bytes);
        self.upload_config = self.upload_config.max_size(size);
        self
    }

    /// Overrides the effective max_size at the form level.
    /// Returns Err if the requested size exceeds the model-defined ceiling.
    fn apply_max_size_bounded(&mut self, size: FileSize) -> Result<(), String> {
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

    /// Maximum number of files. Values > 1 automatically add the `multiple` HTML attribute.
    pub fn max_files(mut self, count: usize) -> Self {
        self.max_files = Some(count);
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

    /// Maximum image dimensions in pixels. Only checked for `FileFieldType::Image`.
    pub fn max_dimensions(mut self, width: u32, height: u32) -> Self {
        self.max_width = Some(width);
        self.max_height = Some(height);
        self
    }

    /// Overrides the extension whitelist. Pass lowercase extensions without dots: `vec!["jpg", "png"]`.
    pub fn allowed_extensions(mut self, exts: Vec<&str>) -> Self {
        self.allowed_extensions = AllowedExtensions::new(exts);
        self
    }
}

#[async_trait]
impl FormField for FileField {
    fn cap_max_size(&mut self, bytes: u64) {
        if self.upload_config.max_size.is_none_or(|s| s > bytes) {
            self.upload_config.max_size = Some(bytes);
        }
    }

    fn model_max_size(&self) -> Option<u64> {
        self.model_max_size
    }

    fn set_max_size_bounded(&mut self, size: FileSize) -> Result<(), String> {
        self.apply_max_size_bounded(size)
    }

    fn set_submitted_value(&mut self, value: &str) {
        let staged = parse_file_list(value).iter().all(|p| is_staged_upload(p));
        if !staged {
            // Not an upload from this request: the client sent a path as text.
            tracing::warn!(field = %self.base.name, "file field: submitted value is not a staged upload, ignored");
            return;
        }
        self.set_value(value);
    }

    fn set_value(&mut self, value: &str) {
        let current = self.base.value.trim().to_string();
        if !current.is_empty() && current != value.trim() {
            self.prev_value = Some(current);
        }
        self.base.value = value.to_string();
    }

    async fn validate(&mut self) -> bool {
        let val = self.base.value.trim();

        // 1. Presence validation
        if self.base.is_required.choice && val.is_empty() {
            let msg = self
                .base
                .is_required
                .message
                .clone()
                .unwrap_or_else(|| t("forms.file_required").to_string());
            self.set_error(msg);
            return false;
        }

        if val.is_empty() {
            self.clear_error();
            return true;
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
            self.base.value.clear();
            self.set_error(tf("forms.file_max_count", &[&max]));
            return false;
        }

        // 3. Extensions + size validation
        for filename in &files {
            if !self.allowed_extensions.is_allowed(filename) {
                let exts = self.allowed_extensions.extensions.join(", ");
                cleanup_files(&files).await;
                self.base.value.clear();
                self.set_error(tf(
                    "forms.file_extension_blocked",
                    &[filename.as_str(), exts.as_str()],
                ));
                return false;
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
                        tracing::warn!(field = %self.base.name, error = %e, "file field: upload size unreadable, refused");
                        cleanup_files(&files).await;
                        self.base.value.clear();
                        self.set_error(t("forms.file_unreadable").to_string());
                        return false;
                    }
                };
                if file_size > max_bytes {
                    let size_mb = file_size as f64 / (1024.0 * 1024.0);
                    let max_mb = max_bytes as f64 / (1024.0 * 1024.0);
                    let size_str = format!("{:.1}", size_mb);
                    let max_mb_str = format!("{:.1}", max_mb);
                    cleanup_files(&files).await;
                    self.base.value.clear();
                    self.set_error(tf(
                        "forms.file_too_large",
                        &[filename.as_str(), size_str.as_str(), max_mb_str.as_str()],
                    ));
                    return false;
                }
            }
        }

        // 4. Image validation: real format + dimensions
        if let FileFieldType::Image = self.field_type {
            for filename in &files {
                if !is_valid_image_content(filename).await {
                    cleanup_files(&files).await;
                    self.base.value.clear();
                    self.set_error(tf("forms.file_invalid_image", &[filename.as_str()]));
                    return false;
                }

                let dims = if self.max_width.is_some() || self.max_height.is_some() {
                    let filename = filename.clone();
                    tokio::task::spawn_blocking(move || {
                        ImageReader::open(&filename).ok()?.into_dimensions().ok()
                    })
                    .await
                    .ok()
                    .flatten()
                } else {
                    None
                };
                if let Some((w, h)) = dims {
                    if let Some(max_w) = self.max_width
                        && w > max_w
                    {
                        let (w_s, mw_s) = (w.to_string(), max_w.to_string());
                        cleanup_files(&files).await;
                        self.base.value.clear();
                        self.set_error(tf(
                            "forms.image_too_wide",
                            &[filename.as_str(), w_s.as_str(), mw_s.as_str()],
                        ));
                        return false;
                    }
                    if let Some(max_h) = self.max_height
                        && h > max_h
                    {
                        let (h_s, mh_s) = (h.to_string(), max_h.to_string());
                        cleanup_files(&files).await;
                        self.base.value.clear();
                        self.set_error(tf(
                            "forms.image_too_tall",
                            &[filename.as_str(), h_s.as_str(), mh_s.as_str()],
                        ));
                        return false;
                    }
                }
            }
        }

        self.clear_error();
        true
    }

    async fn finalize(&mut self) -> Result<(), String> {
        let val = self.base.value.trim().to_string();
        if val.is_empty() {
            return Ok(());
        }

        // Sous-dossier déclaré sur le champ (ex: "plats/"), ou racine MEDIA_ROOT si
        // aucun `upload_to`. `finalize` est le seul committer : un fichier stagé doit
        // toujours être déplacé en destination servie et stocké en chemin RELATIF,
        // y compris sans `upload_to` (sinon le fichier reste hors media_root et la DB
        // garde un chemin absolu de staging).
        let upload_rel_clean = match &self.upload_config.upload_to {
            Some(f) => f(&self.base.name).trim_matches('/').to_string(),
            None => String::new(),
        };

        // Physical destination: {MEDIA_ROOT}/{upload_rel}
        let media_root = resolve_media_root();
        let media_root_clean = media_root.trim_end_matches('/');
        let dest_dir_abs = if upload_rel_clean.is_empty() {
            media_root_clean.to_string()
        } else {
            format!("{}/{}", media_root_clean, upload_rel_clean)
        };
        let dest_dir_abs_path = Path::new(&dest_dir_abs);

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

            // Already in the right physical directory — store as relative path
            if src.parent() == Some(dest_dir_abs_path) {
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

            tokio::fs::create_dir_all(dest_dir_abs_path)
                .await
                .map_err(|e| format!("upload dir '{}': {}", dest_dir_abs, e))?;

            let filename = src
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or(file_path.as_str());
            let dest_abs = dest_dir_abs_path.join(filename);

            move_file(src, &dest_abs)
                .await
                .map_err(|e| format!("move '{}': {}", dest_abs.display(), e))?;

            new_paths.push(to_rel(filename));
        }

        self.base.value = new_paths.join(",");

        // Delete old file(s) if a new upload replaced them
        if let Some(prev) = &self.prev_value {
            for old_rel in prev.split(',').map(|s| s.trim()).filter(|s| !s.is_empty()) {
                if !new_paths.iter().any(|p| p == old_rel) {
                    let old_abs =
                        format!("{}/{}", media_root_clean, old_rel.trim_start_matches('/'));
                    if let Err(e) = tokio::fs::remove_file(&old_abs).await
                        && e.kind() != std::io::ErrorKind::NotFound
                    {
                        tracing::warn!(path = %old_abs, error = %e, "old upload removal failed");
                    }
                }
            }
        }

        Ok(())
    }

    fn render(&self, tera: &ATera) -> Result<String, String> {
        let mut context = self.base_context();

        let is_image = matches!(self.field_type, FileFieldType::Image);

        context.insert("allowed_extensions", &self.allowed_extensions);
        context.insert("multiple", &(self.max_files.unwrap_or(1) > 1));
        context.insert("is_file", &true);
        context.insert("is_image", &is_image);

        if let Some(size) = self.upload_config.max_size {
            context.insert("max_size", &size);
            context.insert("max_size_mb", &(size as f64 / (1024.0 * 1024.0)));
        }
        if let Some(count) = self.max_files {
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
mod finalize_tests {
    use super::*;
    use crate::forms::base::FormField;
    use std::fs;

    fn unique_dir(tag: &str) -> std::path::PathBuf {
        let p = std::env::temp_dir().join(format!("runique_ft_{}_{}", tag, uuid::Uuid::new_v4()));
        fs::create_dir_all(&p).unwrap();
        p
    }

    /// Without `upload_to`, `finalize` must commit the staged file at the root
    /// of MEDIA_ROOT and store a RELATIVE path (not the staging's absolute one).
    /// A prerequisite for `parse_multipart`'s move to an unserved staging area.
    #[tokio::test]
    async fn finalize_without_upload_to_commits_staged_file_to_media_root() {
        let _g = crate::config::static_files::MEDIA_ENV_LOCK.lock().await;
        let media = unique_dir("media");
        // Where `parse_multipart` stages an upload.
        let staging = media.join(format!(".staging-{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&staging).unwrap();
        let staged = staging.join("photo.png");
        fs::write(&staged, b"data").unwrap();

        unsafe {
            std::env::set_var("MEDIA_ROOT", media.to_str().unwrap());
        }

        let mut f = FileField::any("doc");
        f.base.value = staged.to_string_lossy().to_string();

        f.finalize().await.expect("finalize should succeed");

        assert_eq!(f.base.value, "photo.png", "valeur normalisée en relatif");
        assert!(
            media.join("photo.png").exists(),
            "fichier commité en media_root"
        );
        assert!(!staged.exists(), "fichier déplacé hors du staging");

        unsafe {
            std::env::remove_var("MEDIA_ROOT");
        }
        let _ = fs::remove_dir_all(&media);
    }

    /// Stages `photo.png` under a fresh MEDIA_ROOT, runs `finalize` on `field`,
    /// and returns the stored value with the media root to check against.
    async fn finalize_staged(mut field: FileField) -> (String, std::path::PathBuf) {
        let media = unique_dir("media");
        let staging = media.join(format!(".staging-{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&staging).unwrap();
        let staged = staging.join("photo.png");
        fs::write(&staged, b"data").unwrap();
        unsafe {
            std::env::set_var("MEDIA_ROOT", media.to_str().unwrap());
        }
        field.base.value = staged.to_string_lossy().to_string();
        field.finalize().await.expect("finalize should succeed");
        unsafe {
            std::env::remove_var("MEDIA_ROOT");
        }
        (field.base.value, media)
    }

    /// `upload_to_env` is a subdirectory named after the field, under
    /// MEDIA_ROOT — not MEDIA_ROOT nested into itself, which would also put
    /// the server's absolute path into the stored value and the public URL.
    #[tokio::test]
    async fn upload_to_env_commits_under_a_folder_named_after_the_field() {
        let _g = crate::config::static_files::MEDIA_ENV_LOCK.lock().await;
        let (stored, media) = finalize_staged(FileField::any("avatar").upload_to_env()).await;
        assert_eq!(stored, "avatar/photo.png");
        assert!(media.join("avatar/photo.png").exists());
        let _ = fs::remove_dir_all(&media);
    }

    /// `upload_to(&StaticConfig)` is the root of MEDIA_ROOT.
    #[tokio::test]
    async fn upload_to_static_config_commits_at_the_media_root() {
        let _g = crate::config::static_files::MEDIA_ENV_LOCK.lock().await;
        let config = StaticConfig::from_env();
        let (stored, media) = finalize_staged(FileField::any("doc").upload_to(&config)).await;
        assert_eq!(stored, "photo.png");
        assert!(media.join("photo.png").exists());
        let _ = fs::remove_dir_all(&media);
    }

    /// A path the client sent as text is never taken for an upload: `fill`
    /// ignores it, and neither `validate` nor `finalize` touch that file.
    #[tokio::test]
    async fn a_path_that_is_not_an_upload_is_never_touched() {
        let _g = crate::config::static_files::MEDIA_ENV_LOCK.lock().await;
        let media = unique_dir("media3");
        let outside = unique_dir("outside");
        let victim = outside.join("app.db");
        fs::write(&victim, b"server file").unwrap();
        unsafe {
            std::env::set_var("MEDIA_ROOT", media.to_str().unwrap());
        }
        let path = victim.to_string_lossy().to_string();

        let mut submitted = FileField::image("avatar");
        submitted.set_submitted_value(&path);
        assert_eq!(submitted.base.value, "", "ignored when submitted");

        // Set by code instead: rejected by validation, but not deleted.
        let mut by_code = FileField::image("avatar");
        by_code.set_value(&path);
        assert!(!by_code.validate().await);
        assert!(victim.exists(), "a failed validation must not delete it");

        let mut moved = FileField::any("doc");
        moved.base.value = path.clone();
        assert!(moved.finalize().await.is_err(), "not an upload: refused");
        assert!(victim.exists(), "and not moved into MEDIA_ROOT");

        unsafe {
            std::env::remove_var("MEDIA_ROOT");
        }
        let _ = fs::remove_dir_all(&media);
        let _ = fs::remove_dir_all(&outside);
    }

    /// A staged upload is still accepted from the request.
    #[tokio::test]
    async fn a_staged_upload_is_accepted_when_submitted() {
        let _g = crate::config::static_files::MEDIA_ENV_LOCK.lock().await;
        let media = unique_dir("media4");
        let staging = media.join(format!(".staging-{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&staging).unwrap();
        let staged = staging.join("doc.pdf");
        fs::write(&staged, b"%PDF").unwrap();
        unsafe {
            std::env::set_var("MEDIA_ROOT", media.to_str().unwrap());
        }

        let mut f = FileField::document("doc");
        f.set_submitted_value(&staged.to_string_lossy());
        assert_eq!(f.base.value, staged.to_string_lossy());

        unsafe {
            std::env::remove_var("MEDIA_ROOT");
        }
        let _ = fs::remove_dir_all(&media);
    }

    /// Current-flow case: file already at the media_root root → finalize
    /// normalizes it to a relative path without moving it (idempotent).
    #[tokio::test]
    async fn finalize_without_upload_to_normalizes_in_place() {
        let _g = crate::config::static_files::MEDIA_ENV_LOCK.lock().await;
        let media = unique_dir("media2");
        let in_root = media.join("already.png");
        fs::write(&in_root, b"data").unwrap();

        unsafe {
            std::env::set_var("MEDIA_ROOT", media.to_str().unwrap());
        }

        let mut f = FileField::any("doc");
        f.base.value = in_root.to_string_lossy().to_string();

        f.finalize().await.expect("finalize should succeed");

        assert_eq!(f.base.value, "already.png");
        assert!(in_root.exists());

        unsafe {
            std::env::remove_var("MEDIA_ROOT");
        }
        let _ = fs::remove_dir_all(&media);
    }

    /// Current behaviour, not a wish: MEDIA_ROOT is a plain filesystem path.
    /// An address put there reaches no other machine; the upload lands in a
    /// local folder named after it, relative to the working directory. Written
    /// before a startup check exists, so that check has to change this test.
    #[tokio::test]
    async fn an_address_as_media_root_writes_to_a_local_folder_of_that_name() {
        let _g = crate::config::static_files::MEDIA_ENV_LOCK.lock().await;
        let tag = uuid::Uuid::new_v4();
        let cwd = std::env::current_dir().unwrap();

        for (root, top) in [
            (format!("192.0.2.10-{tag}"), format!("192.0.2.10-{tag}")),
            (
                format!("http://192.0.2.10/media-{tag}"),
                "http:".to_string(),
            ),
        ] {
            let top_existed = cwd.join(&top).exists();
            // Staged under MEDIA_ROOT, as `parse_multipart` does.
            let staging = cwd
                .join(&root)
                .join(format!(".staging-{}", uuid::Uuid::new_v4()));
            fs::create_dir_all(&staging).unwrap();
            let staged = staging.join("photo.png");
            fs::write(&staged, b"data").unwrap();

            unsafe { std::env::set_var("MEDIA_ROOT", &root) };
            let mut f = FileField::any("doc");
            f.base.value = staged.to_string_lossy().to_string();
            let result = f.finalize().await;
            unsafe { std::env::remove_var("MEDIA_ROOT") };

            let landed = cwd.join(&root).join("photo.png");
            let written_locally = landed.exists();
            if !top_existed {
                let _ = fs::remove_dir_all(cwd.join(&top));
            }

            assert!(result.is_ok(), "{root}: {result:?}");
            assert_eq!(f.base.value, "photo.png", "{root}");
            assert!(
                written_locally,
                "{root}: expected a local folder under {}",
                cwd.display()
            );
        }
    }
}

/// Written from cargo-mutants survivors (2026-10-02): each test fails when
/// the matching check is altered, not only when the field misbehaves today.
#[cfg(test)]
mod guarantees {
    use super::*;
    use crate::forms::base::FormField;
    use std::fs;
    use std::path::{Path, PathBuf};

    fn temp_dir(tag: &str) -> PathBuf {
        let p = std::env::temp_dir().join(format!("rq_fg_{tag}_{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&p).unwrap();
        p
    }

    /// `MEDIA_ROOT` for the test, with one staging directory inside it.
    struct Media {
        root: PathBuf,
        staging: PathBuf,
    }

    impl Media {
        fn new() -> Self {
            let root = temp_dir("media");
            let staging = root.join(format!(".staging-{}", uuid::Uuid::new_v4()));
            fs::create_dir_all(&staging).unwrap();
            unsafe { std::env::set_var("MEDIA_ROOT", root.to_str().unwrap()) };
            Self { root, staging }
        }

        fn staged(&self, name: &str, bytes: &[u8]) -> String {
            let p = self.staging.join(name);
            fs::write(&p, bytes).unwrap();
            p.to_string_lossy().into_owned()
        }
    }

    impl Drop for Media {
        fn drop(&mut self) {
            unsafe { std::env::remove_var("MEDIA_ROOT") };
            let _ = fs::remove_dir_all(&self.root);
        }
    }

    // ── is_staged_upload ───────────────────────────────────────────────────

    #[tokio::test]
    async fn only_a_file_directly_in_a_staging_dir_under_media_root_is_an_upload() {
        let _g = crate::config::static_files::MEDIA_ENV_LOCK.lock().await;
        let media = Media::new();
        let staged = media.staged("a.png", b"x");
        assert!(is_staged_upload(&staged));

        let at_root = media.root.join("b.png");
        fs::write(&at_root, b"x").unwrap();
        assert!(
            !is_staged_upload(at_root.to_str().unwrap()),
            "in MEDIA_ROOT itself"
        );

        let other_dir = media.root.join("avatars");
        fs::create_dir_all(&other_dir).unwrap();
        fs::write(other_dir.join("c.png"), b"x").unwrap();
        assert!(
            !is_staged_upload(other_dir.join("c.png").to_str().unwrap()),
            "not a staging dir"
        );

        let elsewhere = temp_dir("elsewhere").join(".staging-x");
        fs::create_dir_all(&elsewhere).unwrap();
        fs::write(elsewhere.join("d.png"), b"x").unwrap();
        assert!(
            !is_staged_upload(elsewhere.join("d.png").to_str().unwrap()),
            "staging dir outside MEDIA_ROOT"
        );

        let a_dir = media.staging.join("sub");
        fs::create_dir_all(&a_dir).unwrap();
        assert!(
            !is_staged_upload(a_dir.to_str().unwrap()),
            "a directory isn't a file"
        );
        assert!(!is_staged_upload(
            &media.staging.join("missing.png").to_string_lossy()
        ));
    }

    #[tokio::test]
    async fn an_upload_whose_size_cant_be_read_is_refused() {
        let _g = crate::config::static_files::MEDIA_ENV_LOCK.lock().await;
        let media = Media::new();
        let staged = media.staged("gone.txt", b"1234");
        fs::remove_file(&staged).unwrap();

        let mut field = FileField::any("doc").max_size(FileSize::bytes(8));
        field.set_value(&staged);
        assert!(!field.validate().await, "vanished upload");
        assert!(field.base.value.is_empty(), "finalize must not see it");
        assert!(field.error().is_some());
    }

    #[tokio::test]
    async fn a_path_already_stored_is_not_measured() {
        let _g = crate::config::static_files::MEDIA_ENV_LOCK.lock().await;
        let _media = Media::new();
        let mut field = FileField::any("doc").max_size(FileSize::bytes(8));
        field.set_value("plats/photo.txt");
        assert!(field.validate().await, "kept as is by finalize");
    }

    #[tokio::test]
    async fn cleanup_removes_staged_uploads_only() {
        let _g = crate::config::static_files::MEDIA_ENV_LOCK.lock().await;
        let media = Media::new();
        let staged = media.staged("rejected.png", b"x");
        let kept = media.root.join("kept.png");
        fs::write(&kept, b"x").unwrap();
        cleanup_files(&[staged.clone(), kept.to_string_lossy().into_owned()]).await;
        assert!(
            !Path::new(&staged).exists(),
            "the refused upload is removed"
        );
        assert!(kept.exists(), "anything else stays");
    }

    // ── is_valid_image_content ─────────────────────────────────────────────

    async fn image(name: &str, bytes: &[u8]) -> bool {
        let dir = temp_dir("img");
        let p = dir.join(name);
        fs::write(&p, bytes).unwrap();
        let ok = is_valid_image_content(&p.to_string_lossy()).await;
        let _ = fs::remove_dir_all(&dir);
        ok
    }

    #[tokio::test]
    async fn each_image_format_is_recognised_by_its_magic_bytes() {
        assert!(
            image("a.jpg", &[0xFF, 0xD8, 0xFF, 0xE0, 0, 0, 0, 0, 0, 0, 0, 0]).await,
            "jpeg"
        );
        assert!(
            image("a.png", &[0x89, 0x50, 0x4E, 0x47]).await,
            "png, exactly 4 bytes"
        );
        assert!(image("a.gif", b"GIF89a").await, "gif");
        assert!(
            image("a.webp", b"RIFF\0\0\0\0WEBP").await,
            "webp, exactly 12 bytes"
        );
        assert!(
            image("a.avif", b"\0\0\0\x18ftyp").await,
            "avif/heic, exactly 8 bytes"
        );
    }

    #[tokio::test]
    async fn anything_else_is_not_an_image() {
        assert!(!image("a.png", &[0x89, 0x50, 0x4E]).await, "under 4 bytes");
        assert!(!image("a.png", b"hello world!").await, "text");
        assert!(
            !image("a.wav", b"RIFF\0\0\0\0WAVE").await,
            "RIFF but not WEBP"
        );
        assert!(
            !image("a.webp", b"XXXX\0\0\0\0WEBP").await,
            "WEBP without RIFF"
        );
        assert!(!image("a.mp4", b"\0\0\0\x18ftyq").await, "no ftyp box");
        assert!(
            !image("a.svg", &[0x89, 0x50, 0x4E, 0x47]).await,
            "svg never, whatever its bytes"
        );
        assert!(!image("empty.png", b"").await, "empty");
        assert!(
            !is_valid_image_content("/nonexistent/x.png").await,
            "unreadable"
        );
    }

    // ── Sizes ──────────────────────────────────────────────────────────────

    #[test]
    fn file_sizes_in_bytes() {
        assert_eq!(FileSize::kb(3).as_bytes(), 3 * 1024);
        assert_eq!(FileSize::mb(3).as_bytes(), 3 * 1024 * 1024);
        assert_eq!(FileSize::gb(3).as_bytes(), 3 * 1024 * 1024 * 1024);
        assert_eq!(u64::from(FileSize::kb(2)), 2048);
    }

    #[test]
    fn the_model_ceiling_bounds_form_overrides() {
        let mut field = FileField::any("doc").max_size(FileSize::mb(2));
        assert_eq!(field.model_max_size(), Some(2 * 1024 * 1024));
        let err = field.set_max_size_bounded(FileSize::mb(3)).unwrap_err();
        assert!(err.contains("3.0MB") && err.contains("2.0MB"), "{err}");
        field.set_max_size_bounded(FileSize::mb(1)).unwrap();
        assert_eq!(field.upload_config.max_size, Some(1024 * 1024));
    }

    #[test]
    fn cap_max_size_only_ever_lowers_the_limit() {
        let mut field = FileField::any("doc").max_size(FileSize::bytes(100));
        field.cap_max_size(200);
        assert_eq!(field.upload_config.max_size, Some(100), "never raised");
        field.cap_max_size(100);
        assert_eq!(field.upload_config.max_size, Some(100));
        field.cap_max_size(50);
        assert_eq!(field.upload_config.max_size, Some(50));
        let mut unbounded = FileField::any("doc");
        unbounded.upload_config.max_size = None;
        unbounded.cap_max_size(70);
        assert_eq!(unbounded.upload_config.max_size, Some(70));
    }

    #[tokio::test]
    async fn an_upload_of_exactly_the_limit_is_accepted_one_byte_more_is_not() {
        let _g = crate::config::static_files::MEDIA_ENV_LOCK.lock().await;
        let media = Media::new();
        let mut field = FileField::any("doc").max_size(FileSize::bytes(10));
        field.set_value(&media.staged("ok.txt", &[b'a'; 10]));
        assert!(field.validate().await);
        let mut field = FileField::any("doc").max_size(FileSize::bytes(10));
        field.set_value(&media.staged("big.txt", &[b'a'; 11]));
        assert!(!field.validate().await);
    }

    // ── Previous value and replaced files ─────────────────────────────────

    #[test]
    fn the_previous_value_is_kept_only_when_it_changes() {
        let mut field = FileField::any("doc");
        field.set_value("a.pdf");
        assert_eq!(field.prev_value, None, "nothing before");
        field.set_value("a.pdf");
        assert_eq!(field.prev_value, None, "same value");
        field.set_value("b.pdf");
        assert_eq!(field.prev_value.as_deref(), Some("a.pdf"));
    }

    #[tokio::test]
    async fn a_replaced_file_is_deleted_an_unchanged_one_is_kept() {
        let _g = crate::config::static_files::MEDIA_ENV_LOCK.lock().await;
        let media = Media::new();
        fs::write(media.root.join("old.pdf"), b"old").unwrap();
        let mut field = FileField::any("doc");
        field.set_value("old.pdf");
        field.set_value(&media.staged("new.pdf", b"new"));
        field.finalize().await.unwrap();
        assert!(!media.root.join("old.pdf").exists(), "replaced: removed");
        assert!(media.root.join("new.pdf").exists());

        fs::write(media.root.join("same.pdf"), b"same").unwrap();
        let mut field = FileField::any("doc");
        field.set_value("same.pdf");
        field.prev_value = Some("same.pdf".into());
        field.finalize().await.unwrap();
        assert!(
            media.root.join("same.pdf").exists(),
            "still referenced: kept"
        );
    }

    // ── Rendering ─────────────────────────────────────────────────────────

    #[test]
    fn render_tells_the_template_the_size_and_whether_several_files_are_allowed() {
        let mut tera = tera::Tera::default();
        tera.add_raw_template(
            "base_file.html",
            "{{ max_size_mb }}|{{ multiple }}|{{ is_image }}|{{ max_files | default(value=0) }}",
        )
        .unwrap();
        let tera = std::sync::Arc::new(tera);
        let one = FileField::image("pic").max_size(FileSize::bytes(1_572_864));
        assert_eq!(one.render(&tera).unwrap(), "1.5|false|true|0");
        let several = FileField::document("docs").max_files(3);
        assert_eq!(
            several.render(&tera).unwrap(),
            "10.0|true|false|3",
            "default 10 MB limit, several files"
        );
    }

    #[test]
    fn upload_config_debug_shows_its_settings() {
        let shown = format!("{:?}", FileUploadConfig::default());
        assert!(
            shown.contains("FileUploadConfig") && shown.contains("max_size"),
            "{shown}"
        );
    }
}
