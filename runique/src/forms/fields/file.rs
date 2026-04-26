//! File Field: `FileField` with type, size, and upload path validation.
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
        let _ = std::fs::remove_file(path);
    }
}

/// Checks if the file is a valid image
fn is_valid_path(path: &str) -> bool {
    let p = Path::new(path);
    if !p.exists() {
        return false;
    }
    if path.to_lowercase().ends_with(".svg") {
        return false;
    }
    match ImageReader::open(p) {
        Ok(reader) => reader.with_guessed_format().is_ok(),
        Err(_) => false,
    }
}

/// Parses the field value into a list of file paths
fn parse_file_list(val: &str) -> Vec<&str> {
    val.split(',')
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .collect()
}

#[derive(Debug, Clone, Serialize)]
pub enum FileFieldType {
    None,
    Image,
    Document,
    Any,
}

#[derive(Debug, Clone, Serialize)]
pub struct AllowedExtensions {
    pub extensions: Vec<String>,
}

impl AllowedExtensions {
    pub fn new(extensions: Vec<&str>) -> Self {
        Self {
            extensions: extensions.iter().map(|s| s.to_string()).collect(),
        }
    }

    pub fn images() -> Self {
        Self::new(vec!["jpg", "jpeg", "png", "gif", "webp", "avif"])
    }

    pub fn documents() -> Self {
        Self::new(vec!["pdf", "doc", "docx", "txt", "odt"])
    }

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
    pub fn bytes(n: u64) -> Self { Self(n) }
    pub fn kb(n: u64) -> Self { Self(n * 1024) }
    pub fn mb(n: u64) -> Self { Self(n * 1024 * 1024) }
    pub fn gb(n: u64) -> Self { Self(n * 1024 * 1024 * 1024) }

    pub fn as_bytes(self) -> u64 { self.0 }
}

impl From<FileSize> for u64 {
    fn from(s: FileSize) -> u64 { s.0 }
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

#[derive(Clone, Debug)]
pub struct FileField {
    pub base: FieldConfig,
    pub field_type: FileFieldType,
    pub allowed_extensions: AllowedExtensions,
    pub upload_config: FileUploadConfig,
    pub max_files: Option<usize>,
    pub max_width: Option<u32>,
    pub max_height: Option<u32>,
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
    pub fn create(name: &str, type_field: &str, format: FileFieldType) -> Self {
        let extensions = match format {
            FileFieldType::Image => AllowedExtensions::images(),
            FileFieldType::Document => AllowedExtensions::documents(),
            FileFieldType::Any => AllowedExtensions::any(),
            FileFieldType::None => AllowedExtensions::new(vec![]),
        };

        Self {
            base: FieldConfig::new(name, type_field, "base_file"),
            field_type: format,
            allowed_extensions: extensions,
            upload_config: FileUploadConfig::default(),
            max_files: None,
            max_width: None,
            max_height: None,
        }
    }

    pub fn image(name: &str) -> Self {
        Self::create(name, "file", FileFieldType::Image)
    }

    pub fn document(name: &str) -> Self {
        Self::create(name, "file", FileFieldType::Document)
    }

    pub fn any(name: &str) -> Self {
        Self::create(name, "file", FileFieldType::Any)
    }

    pub fn label(mut self, label: &str) -> Self {
        self.base.label = label.to_string();
        self
    }

    pub fn upload_to(mut self, path: impl IntoUploadPath) -> Self {
        self.upload_config = self.upload_config.upload_to(path.into_upload_path());
        self
    }

    pub fn upload_to_env(mut self) -> Self {
        let media_root = std::env::var("MEDIA_ROOT").unwrap_or_else(|_| "media".to_string());
        let f = Arc::new(move |field_name: &str| format!("{}/{}", media_root, field_name));
        self.upload_config.upload_to = Some(f);
        self
    }

    pub fn max_size(mut self, size: FileSize) -> Self {
        self.upload_config = self.upload_config.max_size(size);
        self
    }

    pub fn max_files(mut self, count: usize) -> Self {
        self.max_files = Some(count);
        if count > 1 {
            self.base
                .html_attributes
                .insert("multiple".to_string(), "multiple".to_string());
        }
        self
    }

    pub fn required(mut self) -> Self {
        self.set_required(true, None);
        self
    }

    pub fn max_dimensions(mut self, width: u32, height: u32) -> Self {
        self.max_width = Some(width);
        self.max_height = Some(height);
        self
    }

    pub fn allowed_extensions(mut self, exts: Vec<&str>) -> Self {
        self.allowed_extensions = AllowedExtensions::new(exts);
        self
    }
}

impl FormField for FileField {
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
        let upload_fn = match &self.upload_config.upload_to {
            Some(f) => f.clone(),
            None => return Ok(()),
        };

        let val = self.base.value.trim().to_string();
        if val.is_empty() {
            return Ok(());
        }

        let dest_dir = upload_fn(&self.base.name);
        let dest_dir_path = Path::new(&dest_dir);

        let files: Vec<String> = val
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        let mut new_paths = Vec::new();

        for file_path in &files {
            let src = Path::new(file_path.as_str());

            // Already in the right directory — nothing to move
            if src.parent() == Some(dest_dir_path) {
                new_paths.push(file_path.clone());
                continue;
            }

            // Not a freshly uploaded file (e.g. stored DB value pointing elsewhere)
            if !src.exists() {
                new_paths.push(file_path.clone());
                continue;
            }

            std::fs::create_dir_all(dest_dir_path)
                .map_err(|e| format!("upload dir '{}': {}", dest_dir, e))?;

            let filename = src
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or(file_path.as_str());
            let dest = dest_dir_path.join(filename);

            std::fs::rename(src, &dest).map_err(|e| format!("move '{}': {}", dest.display(), e))?;

            new_paths.push(dest.to_string_lossy().to_string());
        }

        self.base.value = new_paths.join(",");
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
