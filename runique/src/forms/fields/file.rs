//! File Field: `FileField` with type, size, and upload path validation.
use crate::config::static_files::resolve_media_root;
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

impl IntoUploadPath for &StaticConfig {
    fn into_upload_path(self) -> String {
        self.media_root.clone()
    }
}
use image::ImageReader;
use serde::Serialize;
use std::path::Path;
use std::sync::Arc;
use tera::{Context, Tera};

/// Deletes uploaded files from disk (cleanup on validation failure)
fn cleanup_files(files: &[String]) {
    for path in files {
        if let Err(e) = std::fs::remove_file(path)
            && e.kind() != std::io::ErrorKind::NotFound
        {
            tracing::warn!(path = %path, error = %e, "cleanup_files: remove failed");
        }
    }
}

/// Checks if the file is a valid image using magic bytes.
/// Covers JPEG, PNG, GIF, WebP, and ISO BMFF containers (AVIF, HEIC, HEIF).
fn is_valid_path(path: &str) -> bool {
    use std::io::Read;

    let p = Path::new(path);
    if !p.exists() {
        return false;
    }
    if path.to_lowercase().ends_with(".svg") {
        return false;
    }
    let mut buf = [0u8; 12];
    let n = std::fs::File::open(p)
        .and_then(|mut f| f.read(&mut buf))
        .unwrap_or(0);
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
    pub fn bytes(n: u64) -> Self {
        Self(n)
    }
    pub fn kb(n: u64) -> Self {
        Self(n * 1024)
    }
    pub fn mb(n: u64) -> Self {
        Self(n * 1024 * 1024)
    }
    pub fn gb(n: u64) -> Self {
        Self(n * 1024 * 1024 * 1024)
    }

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
    pub fn new() -> Self {
        Self::default()
    }

    pub fn upload_to(mut self, path: String) -> Self {
        let f = Arc::new(move |_field_name: &str| path.clone());
        self.upload_to = Some(f);
        self
    }

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

    /// Destination directory for uploaded files. Accepts `"media/avatars"`, `String`, or `&StaticConfig`.
    pub fn upload_to(mut self, path: impl IntoUploadPath) -> Self {
        self.upload_config = self.upload_config.upload_to(path.into_upload_path());
        self
    }

    /// Uses `MEDIA_ROOT` from environment as upload root, with subdirectory named after the field.
    pub fn upload_to_env(mut self) -> Self {
        let media_root = resolve_media_root();
        let f = Arc::new(move |field_name: &str| format!("{}/{}", media_root, field_name));
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

impl FormField for FileField {
    fn model_max_size(&self) -> Option<u64> {
        self.model_max_size
    }

    fn set_max_size_bounded(&mut self, size: FileSize) -> Result<(), String> {
        self.apply_max_size_bounded(size)
    }

    fn set_value(&mut self, value: &str) {
        let current = self.base.value.trim().to_string();
        if !current.is_empty() && current != value.trim() {
            self.prev_value = Some(current);
        }
        self.base.value = value.to_string();
    }

    fn validate(&mut self) -> bool {
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
            cleanup_files(&files);
            self.base.value.clear();
            self.set_error(tf("forms.file_max_count", &[&max]));
            return false;
        }

        // 3. Extensions + size validation
        for filename in &files {
            if !self.allowed_extensions.is_allowed(filename) {
                let exts = self.allowed_extensions.extensions.join(", ");
                cleanup_files(&files);
                self.base.value.clear();
                self.set_error(tf(
                    "forms.file_extension_blocked",
                    &[filename.as_str(), exts.as_str()],
                ));
                return false;
            }
            if let Some(max_bytes) = self.upload_config.max_size
                && let Ok(metadata) = std::fs::metadata(filename)
            {
                let file_size = metadata.len();
                if file_size > max_bytes {
                    let size_mb = file_size as f64 / (1024.0 * 1024.0);
                    let max_mb = max_bytes as f64 / (1024.0 * 1024.0);
                    let size_str = format!("{:.1}", size_mb);
                    let max_mb_str = format!("{:.1}", max_mb);
                    cleanup_files(&files);
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
                if !is_valid_path(filename) {
                    cleanup_files(&files);
                    self.base.value.clear();
                    self.set_error(tf("forms.file_invalid_image", &[filename.as_str()]));
                    return false;
                }

                if (self.max_width.is_some() || self.max_height.is_some())
                    && let Ok(reader) = ImageReader::open(filename)
                    && let Ok(dims) = reader.into_dimensions()
                {
                    let (w, h) = dims;
                    if let Some(max_w) = self.max_width
                        && w > max_w
                    {
                        let (w_s, mw_s) = (w.to_string(), max_w.to_string());
                        cleanup_files(&files);
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
                        cleanup_files(&files);
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

    fn finalize(&mut self) -> Result<(), String> {
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

            std::fs::create_dir_all(dest_dir_abs_path)
                .map_err(|e| format!("upload dir '{}': {}", dest_dir_abs, e))?;

            let filename = src
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or(file_path.as_str());
            let dest_abs = dest_dir_abs_path.join(filename);

            std::fs::rename(src, &dest_abs)
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
                    if let Err(e) = std::fs::remove_file(&old_abs)
                        && e.kind() != std::io::ErrorKind::NotFound
                    {
                        tracing::warn!(path = %old_abs, error = %e, "old upload removal failed");
                    }
                }
            }
        }

        Ok(())
    }

    fn render(&self, tera: &Arc<Tera>) -> Result<String, String> {
        let mut context = Context::new();

        let is_image = matches!(self.field_type, FileFieldType::Image);

        context.insert("field", &self.base);
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
        context.insert("readonly", &self.to_json_readonly());
        context.insert("disabled", &self.to_json_disabled());
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

    /// Sans `upload_to`, `finalize` doit committer le fichier stagé en racine de
    /// MEDIA_ROOT et stocker un chemin RELATIF (pas l'absolu du staging).
    /// Prérequis du passage de `parse_multipart` vers un staging non servi.
    #[test]
    fn finalize_without_upload_to_commits_staged_file_to_media_root() {
        let _g = crate::config::static_files::MEDIA_ENV_LOCK
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        let media = unique_dir("media");
        let staging = unique_dir("staging");
        let staged = staging.join("photo.png");
        fs::write(&staged, b"data").unwrap();

        unsafe {
            std::env::set_var("MEDIA_ROOT", media.to_str().unwrap());
        }

        let mut f = FileField::any("doc");
        f.base.value = staged.to_string_lossy().to_string();

        f.finalize().expect("finalize should succeed");

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
        let _ = fs::remove_dir_all(&staging);
    }

    /// Cas flux actuel : fichier déjà en racine media_root → finalize normalise en
    /// relatif sans déplacer (idempotent).
    #[test]
    fn finalize_without_upload_to_normalizes_in_place() {
        let _g = crate::config::static_files::MEDIA_ENV_LOCK
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        let media = unique_dir("media2");
        let in_root = media.join("already.png");
        fs::write(&in_root, b"data").unwrap();

        unsafe {
            std::env::set_var("MEDIA_ROOT", media.to_str().unwrap());
        }

        let mut f = FileField::any("doc");
        f.base.value = in_root.to_string_lossy().to_string();

        f.finalize().expect("finalize should succeed");

        assert_eq!(f.base.value, "already.png");
        assert!(in_root.exists());

        unsafe {
            std::env::remove_var("MEDIA_ROOT");
        }
        let _ = fs::remove_dir_all(&media);
    }
}
