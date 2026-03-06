use crate::utils::trad::{t, tf};
use crate::{
    config::StaticConfig,
    forms::base::{CommonFieldConfig, FieldConfig, FormField},
};
use image::ImageReader;
use serde::Serialize;
use std::path::Path;
use std::sync::Arc;
use tera::{Context, Tera};

/// Vérifie si le fichier est une image valide
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

/// Parse la valeur du champ en liste de chemins de fichiers
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
        Self::new(vec!["jpg", "jpeg", "png", "gif", "webp"])
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

/// Limite appliquée si `.max_size_mb()` n'est jamais appelé.
const DEFAULT_MAX_SIZE_MB: u64 = 10;

/// Configuration d'upload de fichier
#[derive(Clone, Serialize)]
pub struct FileUploadConfig {
    #[serde(skip_serializing)]
    pub upload_to: UploadPathFn,
    pub max_size_mb: Option<u64>,
}

impl Default for FileUploadConfig {
    fn default() -> Self {
        Self {
            upload_to: None,
            max_size_mb: Some(DEFAULT_MAX_SIZE_MB),
        }
    }
}

impl std::fmt::Debug for FileUploadConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FileUploadConfig")
            .field("upload_to", &self.upload_to.as_ref().map(|_| "Fn(...)"))
            .field("max_size_mb", &self.max_size_mb)
            .finish()
    }
}

impl FileUploadConfig {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn upload_to(mut self, media_root: String) -> Self {
        println!("Set upload path to: {}", media_root);
        let f = Arc::new(move |field_name: &str| format!("{}/{}", media_root, field_name));
        self.upload_to = Some(f);
        self
    }

    pub fn max_size_mb(mut self, size: u64) -> Self {
        self.max_size_mb = Some(size);
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

    pub fn upload_to(mut self, cfg: &StaticConfig) -> Self {
        self.upload_config = self.upload_config.upload_to(cfg.media_root.clone());
        self
    }

    pub fn max_size_mb(mut self, size: u64) -> Self {
        self.upload_config = self.upload_config.max_size_mb(size);
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

        // 1. Validation de présence
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

        let files = parse_file_list(val);

        // 2. Validation du nombre de fichiers
        if let Some(max) = self.max_files {
            if files.len() > max {
                self.set_error(tf("forms.file_max_count", &[&max]));
                return false;
            }
        }

        // 3. Validation extensions + taille
        for filename in &files {
            if !self.allowed_extensions.is_allowed(filename) {
                let exts = self.allowed_extensions.extensions.join(", ");
                self.set_error(tf(
                    "forms.file_extension_blocked",
                    &[*filename, exts.as_str()],
                ));
                return false;
            }
            if let Some(max_mb) = self.upload_config.max_size_mb {
                if let Ok(metadata) = std::fs::metadata(filename) {
                    let size_mb = metadata.len() as f64 / (1024.0 * 1024.0);
                    if size_mb > max_mb as f64 {
                        let size_str = format!("{:.1}", size_mb);
                        let max_mb_str = max_mb.to_string();
                        self.set_error(tf(
                            "forms.file_too_large",
                            &[*filename, size_str.as_str(), max_mb_str.as_str()],
                        ));
                        return false;
                    }
                }
            }
        }

        // 4. Validation image : format réel + dimensions
        if let FileFieldType::Image = self.field_type {
            for filename in &files {
                if !is_valid_path(filename) {
                    self.set_error(tf("forms.file_invalid_image", &[*filename]));
                    return false;
                }

                if self.max_width.is_some() || self.max_height.is_some() {
                    if let Ok(reader) = ImageReader::open(filename) {
                        if let Ok(dims) = reader.into_dimensions() {
                            let (w, h) = dims;
                            if let Some(max_w) = self.max_width {
                                if w > max_w {
                                    let (w_s, mw_s) = (w.to_string(), max_w.to_string());
                                    self.set_error(tf(
                                        "forms.image_too_wide",
                                        &[*filename, w_s.as_str(), mw_s.as_str()],
                                    ));
                                    return false;
                                }
                            }
                            if let Some(max_h) = self.max_height {
                                if h > max_h {
                                    let (h_s, mh_s) = (h.to_string(), max_h.to_string());
                                    self.set_error(tf(
                                        "forms.image_too_tall",
                                        &[*filename, h_s.as_str(), mh_s.as_str()],
                                    ));
                                    return false;
                                }
                            }
                        }
                    }
                }
            }
        }

        self.clear_error();
        true
    }

    fn render(&self, tera: &Arc<Tera>) -> Result<String, String> {
        let mut context = Context::new();

        let is_image = matches!(self.field_type, FileFieldType::Image);

        context.insert("field", &self.base);
        context.insert("allowed_extensions", &self.allowed_extensions);
        context.insert("multiple", &(self.max_files.unwrap_or(1) > 1));
        context.insert("is_file", &true);
        context.insert("is_image", &is_image);

        if let Some(size) = self.upload_config.max_size_mb {
            context.insert("max_size_mb", &size);
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
            .map_err(|e| tf("forms.finalize_error", &[&self.base.template_name, &e.to_string()]).to_string())
    }
}
