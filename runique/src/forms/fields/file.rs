//! File Field: `FileField` with type, size, and upload path validation.
use super::media::{UploadCheck, UploadMedia};
use crate::config::static_files::resolve_media_root;
use crate::utils::aliases::ATera;
use crate::utils::trad::tf;
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
use image::{ImageFormat, ImageReader};
use serde::Serialize;
use std::path::{Component, Path, PathBuf};
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

pub(crate) async fn cleanup_files(files: &[String]) {
    for path in files.iter().filter(|p| is_staged_upload(p)) {
        if let Err(e) = tokio::fs::remove_file(path).await
            && e.kind() != std::io::ErrorKind::NotFound
        {
            tracing::warn!(path = %path, error = %e, "cleanup_files: remove failed");
        }
    }
}

pub(crate) fn checked_upload_relative(raw: &str) -> Result<String, String> {
    if raw.is_empty() {
        return Ok(String::new());
    }
    if raw.contains('\\') || raw.contains(':') {
        return Err(format!("invalid upload_to path '{raw}'"));
    }
    let segments = raw.strip_suffix('/').unwrap_or(raw).split('/');
    if segments
        .clone()
        .any(|part| part.is_empty() || matches!(part, "." | ".."))
    {
        return Err(format!("invalid upload_to path '{raw}'"));
    }

    let path = Path::new(raw);
    if path.is_absolute()
        || path
            .components()
            .any(|component| !matches!(component, Component::Normal(_)))
    {
        return Err(format!("invalid upload_to path '{raw}'"));
    }

    Ok(raw.trim_end_matches('/').to_string())
}

pub(crate) async fn ensure_safe_upload_dir(root: &Path, parent: &Path) -> Result<PathBuf, String> {
    let relative = parent.strip_prefix(root).map_err(|_| {
        format!(
            "upload destination '{}' escapes MEDIA_ROOT",
            parent.display()
        )
    })?;
    let mut current = root.to_path_buf();

    for component in relative.components() {
        let Component::Normal(name) = component else {
            return Err(format!("invalid upload destination '{}'", parent.display()));
        };
        current.push(name);
        match tokio::fs::symlink_metadata(&current).await {
            Ok(metadata) if metadata.is_dir() && !metadata.file_type().is_symlink() => {}
            Ok(_) => {
                return Err(format!(
                    "upload destination '{}' contains a non-directory or symlink",
                    current.display()
                ));
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                tokio::fs::create_dir(&current)
                    .await
                    .map_err(|e| format!("upload dir '{}': {e}", current.display()))?;
                let metadata = tokio::fs::symlink_metadata(&current)
                    .await
                    .map_err(|e| format!("upload dir '{}': {e}", current.display()))?;
                if !metadata.is_dir() || metadata.file_type().is_symlink() {
                    return Err(format!(
                        "upload destination '{}' is not a real directory",
                        current.display()
                    ));
                }
            }
            Err(error) => {
                return Err(format!("upload dir '{}': {error}", current.display()));
            }
        }

        let canonical = tokio::fs::canonicalize(&current)
            .await
            .map_err(|e| format!("upload dir '{}': {e}", current.display()))?;
        if !canonical.starts_with(root) {
            return Err(format!(
                "upload destination '{}' escapes MEDIA_ROOT",
                current.display()
            ));
        }
    }

    Ok(current)
}

/// A submitted form that did not reach `finalize`. Removes the staged uploads
/// its file and binary fields still hold, and clears those values. A text
/// field is left as it is, even when its text contains a comma. A path that
/// is not a staged upload — a file already stored — is left on disk and in
/// the field.
pub(crate) async fn discard_staged_uploads(fields: &mut crate::utils::aliases::FieldsMap) {
    for field in fields.values_mut() {
        let kind = field.field_type().to_string();
        match kind.as_str() {
            "file" => discard_file_value(field.as_mut()).await,
            "binary" => discard_binary_value(field.as_mut()).await,
            _ => {}
        }
    }
}

async fn discard_file_value(field: &mut dyn FormField) {
    let value = field.value().to_string();
    let parts = parse_file_list(&value);
    let staged: Vec<String> = parts
        .iter()
        .copied()
        .filter(|p| is_staged_upload(p))
        .map(str::to_string)
        .collect();
    if staged.is_empty() {
        return;
    }
    cleanup_files(&staged).await;
    let kept: Vec<&str> = parts
        .into_iter()
        .filter(|part| !staged.iter().any(|staged| staged == *part))
        .collect();
    field.set_value(&kept.join(","));
}

async fn discard_binary_value(field: &mut dyn FormField) {
    let value = field.value().trim().to_string();
    if value.is_empty() || !is_staged_upload(&value) {
        return;
    }
    cleanup_files(std::slice::from_ref(&value)).await;
    field.set_value("");
}

/// Checks if the file is a valid image using magic bytes.
/// Covers JPEG, PNG, GIF, WebP, and ISO BMFF containers (AVIF, HEIC, HEIF).
pub(crate) async fn is_valid_image_content(path: &str) -> bool {
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

/// Width and height, or why the crate could not measure the file.
/// The text stays in the log: the form shows `forms.file_invalid_image`.
pub(crate) fn read_image_dimensions(path: &str) -> Result<(u32, u32), String> {
    let reader = ImageReader::open(path).map_err(|e| e.to_string())?;
    // `image` only reads AVIF through the system library dav1d: its header alone
    // holds the size, and a project shouldn't have to install anything for it.
    if reader.format() == Some(ImageFormat::Avif) {
        let size = imagesize::size(path).map_err(|e| e.to_string())?;
        let side = |n: usize| u32::try_from(n).map_err(|e| e.to_string());
        return Ok((side(size.width)?, side(size.height)?));
    }
    reader.into_dimensions().map_err(|e| e.to_string())
}

/// Parses the field value into a list of file paths
pub(crate) fn parse_file_list(val: &str) -> Vec<&str> {
    val.split(',')
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .collect()
}

/// Determines which validation rules and allowed extensions apply to a [`FileField`].
#[derive(Debug, Clone, Serialize)]
pub enum FileFieldType {
    None,
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

    /// Preset for audio: mp3, ogg, oga, wav, m4a, flac, aac.
    pub fn audio() -> Self {
        Self::new(vec!["mp3", "ogg", "oga", "wav", "m4a", "flac", "aac"])
    }

    /// Preset for video: mp4, webm, mov, ogv.
    pub fn video() -> Self {
        Self::new(vec!["mp4", "webm", "mov", "ogv"])
    }

    /// Preset for documents: pdf, doc, docx, txt, odt.
    pub fn documents() -> Self {
        Self::new(vec!["pdf", "doc", "docx", "txt", "odt"])
    }

    /// No extension filter. Every extension is accepted except those a browser
    /// runs as a page (`.svg`, `.html`, `.js`…, see [`is_allowed`](Self::is_allowed)).
    /// The content is not inspected: that check belongs to the caller.
    pub fn any() -> Self {
        Self { extensions: vec![] }
    }

    /// Returns whether `filename`'s extension passes this whitelist.
    /// Always rejects what a browser runs as a page of the site once served
    /// from the media root ([`ACTIVE_CONTENT`]), whatever the list; an empty
    /// list allows any other extension.
    pub fn is_allowed(&self, filename: &str) -> bool {
        let ext = Path::new(filename)
            .extension()
            .and_then(|ext| ext.to_str())
            .map(str::to_lowercase);
        if ext
            .as_deref()
            .is_some_and(|ext| ACTIVE_CONTENT.contains(&ext))
        {
            return false;
        }
        if self.extensions.is_empty() {
            return true;
        }
        ext.is_some_and(|ext| {
            self.extensions
                .iter()
                .any(|allowed| allowed.to_lowercase() == ext)
        })
    }
}

/// Extensions a browser opens as a document of the site's origin (scripts,
/// forms, redirects): never accepted by a file field, whatever its list says.
pub const ACTIVE_CONTENT: [&str; 10] = [
    "svg", "svgz", "html", "htm", "xhtml", "xht", "shtml", "xml", "js", "mjs",
];

impl AllowedExtensions {}

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

/// File upload field for documents and other files: [`::document`](FileField::document),
/// [`::any`](FileField::any). Validates extension, size and count; the content
/// is not inspected. Images, audio and video are [`MediaField`](super::MediaField)s.
#[derive(Clone, Debug)]
pub struct FileField {
    pub base: FieldConfig,
    pub field_type: FileFieldType,
    pub upload: UploadMedia,
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
            FileFieldType::Document => AllowedExtensions::documents(),
            FileFieldType::Any => AllowedExtensions::any(),
            FileFieldType::None => AllowedExtensions::new(vec![]),
        };

        Self {
            base: FieldConfig::new(name, type_field, "base_file.html"),
            field_type: format,
            upload: UploadMedia::new(extensions),
        }
    }

    /// Document upload field. Accepts pdf, doc, docx, txt, odt.
    pub fn document(name: &str) -> Self {
        Self::create(name, "file", FileFieldType::Document)
    }

    /// Generic upload field. Every extension is accepted except those a browser
    /// runs as a page ([`ACTIVE_CONTENT`]: `.svg`, `.html`, `.js`…).
    /// The framework does not inspect the content: what the file is, and how
    /// it is served, is the application's responsibility. Use [`image`](Self::image),
    /// [`document`](Self::document), or [`allowed_extensions`](Self::allowed_extensions)
    /// for a closed list.
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
        self.upload.upload_config =
            std::mem::take(&mut self.upload.upload_config).upload_to(path.into_upload_path());
        self
    }

    /// A subdirectory named after the field, under `MEDIA_ROOT`
    /// (`"avatar"` → `{MEDIA_ROOT}/avatar/`).
    pub fn upload_to_env(mut self) -> Self {
        let f = Arc::new(|field_name: &str| field_name.to_string());
        self.upload.upload_config.upload_to = Some(f);
        self
    }

    /// Maximum upload size. Use `FileSize::mb(5)`, `::kb(512)`, etc. Default: 10 MB.
    pub fn max_size(mut self, size: FileSize) -> Self {
        self.upload.set_max_size(size);
        self
    }

    /// Maximum number of files. Values > 1 automatically add the `multiple` HTML attribute.
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

    /// Overrides the extension whitelist. Pass lowercase extensions without dots: `vec!["jpg", "png"]`.
    pub fn allowed_extensions(mut self, exts: Vec<&str>) -> Self {
        self.upload.allowed_extensions = AllowedExtensions::new(exts);
        self
    }
}

#[async_trait]
impl FormField for FileField {
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
            tracing::warn!(field = %self.base.name, "file field: submitted value is not a staged upload, ignored");
            return;
        }
        self.set_value(value);
    }

    fn set_value(&mut self, value: &str) {
        self.upload.track_value(&mut self.base, value);
    }

    async fn validate(&mut self) -> bool {
        // A document or any file: extension, size and count only — the
        // content is the application's to judge (see `MediaField` for one
        // checked by its content).
        match self.upload.check(&self.base).await {
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
            UploadCheck::Files(_) => {}
        }

        self.clear_error();
        true
    }

    async fn finalize(&mut self) -> Result<(), String> {
        self.upload
            .finalize(&mut self.base)
            .await
            .map_err(super::media::upload_failed)
    }

    async fn publish_uploads(&mut self) -> Result<(), String> {
        self.upload
            .publish()
            .await
            .map_err(super::media::upload_failed)
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
        context.insert("is_image", &false);

        if let Some(size) = self.upload.upload_config.max_size {
            context.insert("max_size", &size);
            context.insert("max_size_mb", &(size as f64 / (1024.0 * 1024.0)));
        }
        if let Some(count) = self.upload.max_files {
            context.insert("max_files", &count);
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
    use crate::forms::fields::MediaField;
    use std::fs;

    fn unique_dir(tag: &str) -> std::path::PathBuf {
        let p = std::env::temp_dir().join(format!("runique_ft_{}_{}", tag, uuid::Uuid::new_v4()));
        fs::create_dir_all(&p).unwrap();
        p
    }

    struct Media {
        root: PathBuf,
        staging: PathBuf,
    }

    impl Media {
        fn new() -> Self {
            let root = unique_dir("media");
            let staging = root.join(format!(".staging-{}", uuid::Uuid::new_v4()));
            fs::create_dir_all(&staging).unwrap();
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
            crate::config::static_files::restore_media_root();
            let _ = fs::remove_dir_all(&self.root);
        }
    }

    /// `finalize` prepares the relative path, while publication is a separate
    /// step that can be rolled back if the database write fails.
    #[tokio::test]
    async fn finalize_prepares_but_does_not_publish_staged_file() {
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
        assert!(staged.exists(), "finalize leaves the file staged");
        assert!(!media.join("photo.png").exists(), "not yet published");

        f.publish_uploads()
            .await
            .expect("publication should succeed");
        assert!(
            media.join("photo.png").exists(),
            "published under media_root"
        );
        assert!(!staged.exists(), "staged link removed after publication");

        crate::config::static_files::restore_media_root();
        let _ = fs::remove_dir_all(&media);
    }

    /// A publication that fails (here a file where the folder should be) tells
    /// the visitor a plain sentence: no server path, no OS error.
    #[tokio::test]
    async fn a_failed_publication_shows_no_server_detail() {
        let _g = crate::config::static_files::MEDIA_ENV_LOCK.lock().await;
        let media = unique_dir("media");
        let staging = media.join(format!(".staging-{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&staging).unwrap();
        let staged = staging.join("photo.png");
        fs::write(&staged, b"data").unwrap();
        unsafe {
            std::env::set_var("MEDIA_ROOT", media.to_str().unwrap());
        }
        fs::write(media.join("avatar"), b"not a folder").unwrap();
        let mut f = FileField::any("avatar").upload_to_env();
        f.base.value = staged.to_string_lossy().to_string();
        let error = match f.finalize().await {
            Err(error) => error,
            Ok(()) => f.publish_uploads().await.unwrap_err(),
        };
        assert_eq!(error, crate::utils::trad::t("forms.upload_failed"));
        assert!(!error.contains(media.to_str().unwrap()), "{error}");

        crate::config::static_files::restore_media_root();
        let _ = fs::remove_dir_all(&media);
    }

    /// Stages `photo.png`, prepares and publishes it, then returns its stored
    /// value and media root.
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
        field
            .publish_uploads()
            .await
            .expect("publication should succeed");
        field.complete_uploads().await;
        crate::config::static_files::restore_media_root();
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

        let mut submitted = MediaField::image("avatar");
        submitted.set_submitted_value(&path);
        assert_eq!(submitted.base.value, "", "ignored when submitted");

        // Set by code instead: rejected by validation, but not deleted.
        let mut by_code = MediaField::image("avatar");
        by_code.set_value(&path);
        assert!(!by_code.validate().await);
        assert!(victim.exists(), "a failed validation must not delete it");

        let mut moved = FileField::any("doc");
        moved.base.value = path.clone();
        assert!(moved.finalize().await.is_err(), "not an upload: refused");
        assert!(victim.exists(), "and not moved into MEDIA_ROOT");

        crate::config::static_files::restore_media_root();
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

        crate::config::static_files::restore_media_root();
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

        crate::config::static_files::restore_media_root();
        let _ = fs::remove_dir_all(&media);
    }

    #[tokio::test]
    async fn rollback_removes_new_file_and_preserves_previous_file() {
        let _g = crate::config::static_files::MEDIA_ENV_LOCK.lock().await;
        let media = Media::new();
        fs::write(media.root.join("old.png"), b"old bytes").unwrap();
        let staged = media.staged("new.png", b"new bytes");
        let mut field = FileField::any("doc");
        field.set_value("old.png");
        field.set_value(&staged);

        field.finalize().await.unwrap();
        assert!(Path::new(&staged).exists());
        assert!(media.root.join("old.png").exists());
        field.publish_uploads().await.unwrap();
        assert!(media.root.join("new.png").exists());
        assert!(media.root.join("old.png").exists());

        field.rollback_uploads().await;
        assert!(!media.root.join("new.png").exists());
        assert!(media.root.join("old.png").exists());
        assert_eq!(field.base.value, "old.png");
    }

    #[tokio::test]
    async fn a_refused_write_never_moves_the_upload() {
        let _g = crate::config::static_files::MEDIA_ENV_LOCK.lock().await;
        let media = Media::new();
        fs::write(media.root.join("old.png"), b"old bytes").unwrap();
        let staged = media.staged("new.png", b"new bytes");
        let mut field = FileField::any("doc");
        field.set_value("old.png");
        field.set_value(&staged);
        assert_eq!(field.upload.prev_value.as_deref(), Some("old.png"));
        field.finalize().await.unwrap();

        let mut forms = crate::forms::form::Forms::new("");
        forms.field(&field);

        let result = crate::forms::form::write_with_uploads(&mut forms, async {
            Err(sea_orm::DbErr::Custom(
                "simulated write failure".to_string(),
            ))
        })
        .await;

        assert!(result.is_err());
        assert!(!media.root.join("new.png").exists());
        assert!(Path::new(&staged).exists(), "left to the staging cleanup");
        assert!(media.root.join("old.png").exists());
        assert_eq!(forms.fields["doc"].value(), "old.png");
    }

    /// Once the write committed, the upload is settled: a rollback asked
    /// afterwards (a later failure in the handler) neither removes the moved
    /// file nor brings the previous value back.
    #[tokio::test]
    async fn a_completed_upload_is_no_longer_taken_back() {
        let _g = crate::config::static_files::MEDIA_ENV_LOCK.lock().await;
        let media = Media::new();
        let staged = media.staged("new.png", b"new bytes");
        let mut field = FileField::any("doc");
        field.set_value("old.png");
        field.set_value(&staged);
        field.finalize().await.unwrap();

        let mut forms = crate::forms::form::Forms::new("");
        forms.field(&field);
        forms.publish_uploads().await.unwrap();
        forms.complete_uploads().await;
        forms.rollback_uploads().await;

        assert!(media.root.join("new.png").exists(), "settled: kept");
        assert_eq!(forms.fields["doc"].value(), "new.png");
    }

    #[tokio::test]
    async fn the_upload_moves_after_the_write_and_before_the_commit() {
        use sea_orm::TransactionTrait;
        let _g = crate::config::static_files::MEDIA_ENV_LOCK.lock().await;
        let media = Media::new();
        fs::write(media.root.join("old.png"), b"old bytes").unwrap();
        let staged = media.staged("new.png", b"new bytes");
        let mut field = FileField::any("doc");
        field.set_value("old.png");
        field.set_value(&staged);
        field.finalize().await.unwrap();

        let mut forms = crate::forms::form::Forms::new("");
        forms.field(&field);
        let db = sea_orm::Database::connect("sqlite::memory:").await.unwrap();
        let new = media.root.join("new.png");

        let result = crate::forms::form::write_with_uploads(&mut forms, async {
            let txn = db.begin().await?;
            assert!(!new.exists(), "not moved before the write");
            Ok(txn)
        })
        .await;

        assert!(result.is_ok());
        assert!(new.exists());
        assert!(!Path::new(&staged).exists());
        assert!(
            media.root.join("old.png").exists(),
            "the replaced file stays"
        );
        assert_eq!(forms.fields["doc"].value(), "new.png");
    }

    #[tokio::test]
    async fn completion_keeps_the_replaced_file() {
        let _g = crate::config::static_files::MEDIA_ENV_LOCK.lock().await;
        let media = Media::new();
        fs::write(media.root.join("old.png"), b"old bytes").unwrap();
        let staged = media.staged("new.png", b"new bytes");
        let mut field = FileField::any("doc");
        field.set_value("old.png");
        field.set_value(&staged);
        field.finalize().await.unwrap();

        assert!(media.root.join("old.png").exists());
        field.publish_uploads().await.unwrap();
        assert!(media.root.join("old.png").exists());
        assert!(media.root.join("new.png").exists());
        field.complete_uploads().await;
        assert!(media.root.join("old.png").exists());
        assert!(media.root.join("new.png").exists());
        assert_eq!(field.upload.prev_value, None);
    }

    #[tokio::test]
    async fn upload_to_rejects_traversal_absolute_and_windows_paths() {
        let _g = crate::config::static_files::MEDIA_ENV_LOCK.lock().await;
        let media = Media::new();
        for upload_to in [
            "../outside",
            "avatars/../../outside",
            "/tmp/outside",
            r"C:\outside",
        ] {
            let staged = media.staged(&format!("{}.png", uuid::Uuid::new_v4()), b"data");
            let mut field = FileField::any("doc").upload_to(upload_to);
            field.base.value = staged;
            assert!(
                field.finalize().await.is_err(),
                "rejected upload_to {upload_to:?}"
            );
        }
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn upload_to_symlink_cannot_escape_media_root() {
        use std::os::unix::fs::symlink;

        let _g = crate::config::static_files::MEDIA_ENV_LOCK.lock().await;
        let media = Media::new();
        let outside = unique_dir("outside");
        symlink(&outside, media.root.join("avatars")).unwrap();
        let staged = media.staged("photo.png", b"data");
        let mut field = FileField::any("doc").upload_to("avatars");
        field.base.value = staged.clone();
        field.finalize().await.unwrap();
        assert!(field.publish_uploads().await.is_err());
        assert!(!outside.join("photo.png").exists());
        assert!(Path::new(&staged).exists());
        let _ = fs::remove_dir_all(outside);
    }

    #[tokio::test]
    async fn publication_collision_never_overwrites_existing_file() {
        let _g = crate::config::static_files::MEDIA_ENV_LOCK.lock().await;
        let media = Media::new();
        fs::write(media.root.join("photo.png"), b"existing").unwrap();
        let staged = media.staged("photo.png", b"uploaded");
        let mut field = FileField::any("doc");
        field.base.value = staged.clone();
        field.finalize().await.unwrap();

        assert!(field.publish_uploads().await.is_err());
        assert_eq!(fs::read(media.root.join("photo.png")).unwrap(), b"existing");
        assert!(Path::new(&staged).exists());
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
            if result.is_ok() {
                f.publish_uploads().await.unwrap();
                f.complete_uploads().await;
            }
            crate::config::static_files::restore_media_root();

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
    use crate::forms::field::{FormAccess, RuniqueForm};
    use crate::forms::fields::MediaField;
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
            crate::config::static_files::restore_media_root();
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

        let outside = temp_dir("elsewhere");
        let elsewhere = outside.join(".staging-x");
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
        let _ = fs::remove_dir_all(&outside);
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

    fn encode_image(format: image::ImageFormat, w: u32, h: u32) -> Vec<u8> {
        let img = image::DynamicImage::ImageRgb8(image::RgbImage::from_pixel(
            w,
            h,
            image::Rgb([9, 8, 7]),
        ));
        let mut buf = std::io::Cursor::new(Vec::new());
        img.write_to(&mut buf, format).expect("encode");
        buf.into_inner()
    }

    /// `true` when a staged image passes `max_dimensions`. A refusal must drop
    /// the value and the staged file, same as the other file checks.
    async fn image_with_limit(name: &str, bytes: &[u8], max_w: u32, max_h: u32) -> bool {
        let _g = crate::config::static_files::MEDIA_ENV_LOCK.lock().await;
        let media = Media::new();
        let staged = media.staged(name, bytes);
        let mut field = MediaField::image("photo").max_dimensions(max_w, max_h);
        field.set_value(&staged);
        let ok = field.validate().await;
        if !ok {
            assert!(field.base.value.is_empty(), "{name}");
            assert!(field.error().is_some(), "{name}");
            assert!(
                !Path::new(&staged).exists(),
                "{name}: the refused upload is removed"
            );
        }
        ok
    }

    #[tokio::test]
    async fn a_dimension_limit_measures_every_format_the_crate_decodes() {
        for (format, name) in [
            (image::ImageFormat::Png, "a.png"),
            (image::ImageFormat::Jpeg, "a.jpg"),
            (image::ImageFormat::Gif, "a.gif"),
            (image::ImageFormat::WebP, "a.webp"),
        ] {
            let small = encode_image(format, 2, 2);
            assert!(
                image_with_limit(name, &small, 8, 8).await,
                "{name} within the limit"
            );
            let wide = encode_image(format, 8, 2);
            assert!(
                !image_with_limit(name, &wide, 4, 8).await,
                "{name} over the width"
            );
            let tall = encode_image(format, 2, 8);
            assert!(
                !image_with_limit(name, &tall, 8, 4).await,
                "{name} over the height"
            );
        }
    }

    #[tokio::test]
    async fn a_dimension_limit_refuses_an_image_the_crate_cannot_measure() {
        let samples: &[(&str, &[u8])] = &[
            ("a.jpg", &[0xFF, 0xD8, 0xFF, 0xE0, 0, 0, 0, 0, 0, 0, 0, 0]),
            (
                "a.png",
                &[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0, 0, 0, 0],
            ),
            ("a.gif", b"GIF89a\0\0\0\0\0\0"),
            ("a.webp", b"RIFF\0\0\0\0WEBP"),
            ("a.avif", b"\0\0\0\x18ftypavif"),
            ("clip.avif", b"\0\0\0\x20ftypmp42"),
        ];
        for (name, bytes) in samples {
            assert!(
                !image_with_limit(name, bytes, 64, 64).await,
                "{name} skipped the limit"
            );
        }
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

    /// The byte check stops at `ftyp`. An MP4 carries the same box with another
    /// brand. The `image` crate's own signature asks for `ftypavif`.
    #[tokio::test]
    async fn avif_byte_check_accepts_another_ftyp_brand() {
        let mp4 = b"\0\0\0\x20ftypmp42";
        assert!(
            image("clip.bin", mp4).await,
            "the check does not read the brand"
        );
        assert!(
            image::guess_format(mp4).is_err(),
            "the image crate refuses a brand other than avif"
        );
    }

    /// `image` encodes, `imagesize` reads the header back: no system library. A
    /// file without a readable size is refused, never let past the limit.
    #[tokio::test]
    async fn avif_roundtrip_is_measured_by_the_dimension_limit() {
        let bytes = encode_image(image::ImageFormat::Avif, 8, 2);
        assert!(
            bytes.windows(8).any(|w| w == b"ftypavif"),
            "the encoder writes the avif brand"
        );
        assert_eq!(
            image::guess_format(&bytes).expect("brand avif"),
            image::ImageFormat::Avif
        );

        let dir = temp_dir("avif-read");
        let path = dir.join("photo.avif");
        fs::write(&path, &bytes).unwrap();
        let dims = read_image_dimensions(&path.to_string_lossy()).expect("avif size");
        assert_eq!(dims, (8, 2));
        let _ = fs::remove_dir_all(&dir);

        assert!(
            image_with_limit("photo.avif", &bytes, 8, 8).await,
            "within the limit"
        );
        assert!(
            !image_with_limit("photo.avif", &bytes, 4, 8).await,
            "over the width"
        );
        assert!(
            !image_with_limit("photo.avif", &bytes, 8, 1).await,
            "over the height"
        );
        assert!(
            !image_with_limit("bare.avif", b"\0\0\0\x18ftypavif", 8, 8).await,
            "a header without a size"
        );
        let cut = &bytes[..bytes.len().min(40)];
        assert!(
            !image_with_limit("cut.avif", cut, 8, 8).await,
            "a file cut before its size"
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
        assert_eq!(field.upload.upload_config.max_size, Some(1024 * 1024));
    }

    #[test]
    fn cap_max_size_only_ever_lowers_the_limit() {
        let mut field = FileField::any("doc").max_size(FileSize::bytes(100));
        field.cap_max_size(200);
        assert_eq!(
            field.upload.upload_config.max_size,
            Some(100),
            "never raised"
        );
        field.cap_max_size(100);
        assert_eq!(field.upload.upload_config.max_size, Some(100));
        field.cap_max_size(50);
        assert_eq!(field.upload.upload_config.max_size, Some(50));
        let mut unbounded = FileField::any("doc");
        unbounded.upload.upload_config.max_size = None;
        unbounded.cap_max_size(70);
        assert_eq!(unbounded.upload.upload_config.max_size, Some(70));
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
        assert_eq!(field.upload.prev_value, None, "nothing before");
        field.set_value("a.pdf");
        assert_eq!(field.upload.prev_value, None, "same value");
        field.set_value("b.pdf");
        assert_eq!(field.upload.prev_value.as_deref(), Some("a.pdf"));
    }

    #[tokio::test]
    async fn neither_a_replaced_nor_an_unchanged_file_is_deleted() {
        let _g = crate::config::static_files::MEDIA_ENV_LOCK.lock().await;
        let media = Media::new();
        fs::write(media.root.join("old.pdf"), b"old").unwrap();
        let mut field = FileField::any("doc");
        field.set_value("old.pdf");
        field.set_value(&media.staged("new.pdf", b"new"));
        field.finalize().await.unwrap();
        assert!(
            media.root.join("old.pdf").exists(),
            "still present before save"
        );
        field.publish_uploads().await.unwrap();
        assert!(
            media.root.join("old.pdf").exists(),
            "still present before DB success"
        );
        assert!(media.root.join("new.pdf").exists());
        field.complete_uploads().await;
        assert!(
            media.root.join("old.pdf").exists(),
            "replaced: kept, removing it is the developer's call"
        );

        fs::write(media.root.join("same.pdf"), b"same").unwrap();
        let mut field = FileField::any("doc");
        field.set_value("same.pdf");
        field.upload.prev_value = Some("same.pdf".into());
        field.finalize().await.unwrap();
        field.publish_uploads().await.unwrap();
        field.complete_uploads().await;
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
        let one = MediaField::image("pic").max_size(FileSize::bytes(1_572_864));
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

    // ── A refused form drops what it staged ───────────────────────────────

    #[derive(Clone)]
    struct UploadForm {
        form: crate::forms::form::Forms,
    }

    impl crate::forms::field::FormAccess for UploadForm {
        fn register_fields(form: &mut crate::forms::form::Forms) {
            form.field(&FileField::any("doc"));
            form.field(&crate::forms::fields::BinaryField::new("blob"));
            form.field(&crate::forms::fields::TextField::text("title").required());
            form.field(&crate::forms::fields::TextField::text("note"));
        }
        crate::impl_form_access!();
    }

    impl crate::forms::field::RuniqueForm for UploadForm {}

    fn upload_form() -> UploadForm {
        let mut form = crate::forms::Forms::new("t");
        UploadForm::register_fields(&mut form);
        UploadForm::from_form(form)
    }

    #[tokio::test]
    async fn clearing_a_staged_upload_does_not_remember_it_as_the_old_file() {
        let _g = crate::config::static_files::MEDIA_ENV_LOCK.lock().await;
        let media = Media::new();
        let staged = media.staged("a.txt", b"x");
        let mut field = FileField::any("doc");
        field.set_value(&staged);
        field.set_value("");
        assert!(
            field.upload.prev_value.is_none(),
            "a staging path is not a stored file"
        );
        field.set_value("plats/old.txt");
        field.set_value("plats/new.txt");
        assert_eq!(field.upload.prev_value.as_deref(), Some("plats/old.txt"));
    }

    #[tokio::test]
    async fn a_form_that_was_not_submitted_leaves_the_staged_file() {
        let _g = crate::config::static_files::MEDIA_ENV_LOCK.lock().await;
        let media = Media::new();
        let staged = media.staged("a.txt", b"hello");
        let mut form = upload_form();
        form.forms_mut()
            .fields
            .get_mut("doc")
            .unwrap()
            .set_value(&staged);
        assert!(!form.is_valid().await);
        assert!(Path::new(&staged).exists(), "not a submission");
    }

    #[tokio::test]
    async fn another_field_failing_removes_the_staged_uploads() {
        let _g = crate::config::static_files::MEDIA_ENV_LOCK.lock().await;
        let media = Media::new();
        let doc = media.staged("hello.txt", b"hello");
        let blob = media.staged("blob.bin", b"xyz");
        let mut form = upload_form();
        let fields = form.forms_mut();
        fields.add_value("doc", &doc);
        fields.add_value("blob", &blob);
        fields.add_value("note", "a, b");
        assert!(!form.is_valid().await, "title is required and empty");
        assert!(!Path::new(&doc).exists(), "the file upload is gone");
        assert!(!Path::new(&blob).exists(), "the binary upload is gone");
        assert!(form.forms().fields.get("doc").unwrap().value().is_empty());
        assert!(form.forms().fields.get("blob").unwrap().value().is_empty());
        assert_eq!(form.forms().fields.get("note").unwrap().value(), "a, b");
    }

    #[tokio::test]
    async fn an_invalid_token_removes_the_staged_uploads() {
        let _g = crate::config::static_files::MEDIA_ENV_LOCK.lock().await;
        let media = Media::new();
        let doc = media.staged("hello.txt", b"hello");
        let mut form = upload_form();
        let fields = form.forms_mut();
        fields.force_invalid = true;
        fields.add_value("doc", &doc);
        fields.add_value("title", "ok");
        fields.add_value("note", "a, b");
        assert!(!form.is_valid().await);
        assert!(!Path::new(&doc).exists());
        assert!(form.forms().fields.get("doc").unwrap().value().is_empty());
        assert_eq!(form.forms().fields.get("note").unwrap().value(), "a, b");
        assert_eq!(form.forms().fields.get("title").unwrap().value(), "ok");
    }

    #[tokio::test]
    async fn an_accepted_upload_survives_the_request_dropping_its_staging_dir() {
        let _g = crate::config::static_files::MEDIA_ENV_LOCK.lock().await;
        let media = Media::new();
        let doc = media.staged("hello.txt", b"hello");
        let blob = media.staged("blob.bin", b"xyz");
        let mut form = upload_form();
        let fields = form.forms_mut();
        fields.add_value("doc", &doc);
        fields.add_value("blob", &blob);
        fields.add_value("title", "ok");
        fields.add_value("note", "a, b");
        assert!(form.is_valid().await, "the form validates");
        form.forms_mut().publish_uploads().await.unwrap();
        form.forms_mut().complete_uploads().await;
        let stored = form.forms().fields.get("doc").unwrap().value();
        assert_eq!(stored, "hello.txt");
        assert_eq!(fs::read(media.root.join("hello.txt")).unwrap(), b"hello");
        assert!(!Path::new(&doc).exists(), "published out of staging");
        {
            let mut lease = crate::utils::parse_html::Lease::claim(media.staging.clone());
            let _ = lease.lock_dir();
            let _guard =
                std::sync::Arc::new(crate::utils::parse_html::StagingDir::from_lease(lease));
        }
        assert!(
            !media.staging.exists(),
            "the empty directory goes with the request"
        );
        assert_eq!(fs::read(media.root.join("hello.txt")).unwrap(), b"hello");
        assert_eq!(form.forms().fields.get("note").unwrap().value(), "a, b");
    }
}
