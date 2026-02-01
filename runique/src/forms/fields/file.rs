use crate::forms::field::FormField;
use crate::forms::options::BoolChoice;
use crate::{config::StaticConfig, forms::base::FieldConfig};
use image::ImageReader;
use serde::Serialize;
use serde_json::{json, Value};
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
        // SVG est exclu par défaut ici pour plus de sécurité
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

/// Configuration d’upload de fichier
#[derive(Clone, Serialize)]
pub struct FileUploadConfig {
    #[serde(skip_serializing)]
    pub upload_to: Option<Arc<dyn Fn(&str) -> String + Send + Sync>>,
    pub max_size_mb: Option<u64>,
}

impl std::fmt::Debug for FileUploadConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FileUploadConfig")
            .field("upload_to", &self.upload_to.as_ref().map(|_| "Fn(...)"))
            .field("max_size_mb", &self.max_size_mb)
            .finish()
    }
}

impl Default for FileUploadConfig {
    fn default() -> Self {
        Self {
            upload_to: None,
            max_size_mb: None,
        }
    }
}

impl FileUploadConfig {
    pub fn new() -> Self {
        Self::default()
    }

    /// CORRECTION : Utilise directement la String du media_root pour éviter les conflits de types
    pub fn set_upload_path(mut self, media_root: String) -> Self {
        let f = Arc::new(move |field_name: &str| format!("{}/{}", media_root, field_name));
        self.upload_to = Some(f);
        self
    }

    /// Version originale utilisant l'Arc si vous l'avez déjà
    pub fn upload_to(mut self, cfg: Arc<StaticConfig>) -> Self {
        let root = cfg.media_root.clone();
        let root_for_closure = root.clone();
        let f = Arc::new(move |field_name: &str| format!("{}/{}", root_for_closure, field_name));
        self.upload_to = Some(f);
        println!("Set upload path to: {:?}", root);
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

impl FileField {
    pub fn create(name: &str, type_field: &str, format: FileFieldType) -> Self {
        let extensions = match format {
            FileFieldType::Image => AllowedExtensions::images(),
            FileFieldType::Document => AllowedExtensions::documents(),
            FileFieldType::Any => AllowedExtensions::any(),
            FileFieldType::None => AllowedExtensions::new(vec![]),
        };

        let field = Self {
            base: FieldConfig::new(name, type_field, "base_file"),
            field_type: format,
            allowed_extensions: extensions,
            upload_config: FileUploadConfig::default(),
            max_files: None,
            max_width: None,
            max_height: None,
        };
        field
    }

    // --- Nouveaux Constructeurs publics (comme TextField) ---
    pub fn image(name: &str) -> Self {
        Self::create(name, "file", FileFieldType::Image)
    }

    pub fn document(name: &str) -> Self {
        Self::create(name, "file", FileFieldType::Document)
    }

    pub fn any(name: &str) -> Self {
        Self::create(name, "file", FileFieldType::Any)
    }

    // --- Builder Methods ---
    pub fn label(mut self, label: &str) -> Self {
        self.base.label = label.to_string();
        self
    }

    pub fn upload_to(mut self, cfg: &StaticConfig) -> Self {
        self.upload_config = self.upload_config.set_upload_path(cfg.media_root.clone());
        self
    }

    pub fn max_size_mb(mut self, size: u64) -> Self {
        self.upload_config = self.upload_config.max_size_mb(size);
        self
    }

    pub fn max_files(mut self, count: usize) -> Self {
        self.max_files = Some(count);
        // On ajoute automatiquement l'attribut HTML multiple si count > 1
        if count > 1 {
            self.base
                .html_attributes
                .insert("multiple".to_string(), "multiple".to_string());
        }
        self
    }

    pub fn required(mut self, msg: &str) -> Self {
        self.set_required(true, Some(msg));
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
                .unwrap_or_else(|| "Veuillez sélectionner au moins un fichier".into());
            self.set_error(msg);
            return false;
        }

        if !val.is_empty() {
            let files: Vec<&str> = val
                .split(',')
                .map(|s| s.trim())
                .filter(|s| !s.is_empty())
                .collect();

            // 2. Validation du nombre de fichiers
            if let Some(max) = self.max_files {
                if files.len() > max {
                    self.set_error(format!("Maximum {} fichiers autorisés", max));
                    return false;
                }
            }

            // 3. Validation des extensions
            for filename in files {
                if !self.allowed_extensions.is_allowed(filename) {
                    let exts = self.allowed_extensions.extensions.join(", ");
                    self.set_error(format!(
                        "Fichier '{}' non autorisé. Extensions: {}",
                        filename, exts
                    ));
                    return false;
                }
            }
        }

        // 4. Validation des images si applicable
        if let FileFieldType::Image = self.field_type {
            let files: Vec<&str> = val
                .split(',')
                .map(|s| s.trim())
                .filter(|s| !s.is_empty())
                .collect();
            for filename in files {
                if !is_valid_path(filename) {
                    self.set_error(format!(
                        "Le fichier '{}' n'est pas une image valide",
                        filename
                    ));
                    return false;
                }

                // 5. Validation des dimensions (AJOUT)
                if self.max_width.is_some() || self.max_height.is_some() {
                    if let Ok(reader) = ImageReader::open(filename) {
                        if let Ok(dims) = reader.into_dimensions() {
                            let (w, h) = dims;
                            if let Some(max_w) = self.max_width {
                                if w > max_w {
                                    self.set_error(format!(
                                        "L'image '{}' est trop large ({}px > {}px)",
                                        filename, w, max_w
                                    ));
                                    return false;
                                }
                            }
                            if let Some(max_h) = self.max_height {
                                if h > max_h {
                                    self.set_error(format!(
                                        "L'image '{}' est trop haute ({}px > {}px)",
                                        filename, h, max_h
                                    ));
                                    return false;
                                }
                            }
                        }
                    }
                }
            }
        }
        match &self.field_type {
            FileFieldType::Document | FileFieldType::Any | FileFieldType::None => {
                // Pas de validation supplémentaire pour les documents ou tout type
            }
            FileFieldType::Image => {
                // 4. Validation des images
                let files: Vec<&str> = val
                    .split(',')
                    .map(|s| s.trim())
                    .filter(|s| !s.is_empty())
                    .collect();
                for filename in files {
                    if !is_valid_path(filename) {
                        self.set_error(format!(
                            "Le fichier '{}' n'est pas une image valide",
                            filename
                        ));
                        return false;
                    }
                }
            }
        }

        self.set_error("".into());
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

        // Paramètres de contraintes
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

        tera.render(&self.base.template_name, &context)
            .map_err(|e| format!("Erreur Tera (FileField): {}", e))
    }

    // AJOUT : Méthodes JSON manquantes pour satisfaire le trait FormField
    fn to_json_readonly(&self) -> Value {
        // Les FileFields n'ont souvent pas de config.readonly propre,
        // on retourne une structure par défaut ou on l'ajoute à la struct FileField si nécessaire
        json!({"choice": false, "message": null})
    }

    fn to_json_disabled(&self) -> Value {
        json!({"choice": false, "message": null})
    }

    // Méthodes déjà présentes mais pour rappel de cohérence
    fn to_json_value(&self) -> Value {
        json!(self
            .base
            .value
            .split(',')
            .filter(|s| !s.is_empty())
            .collect::<Vec<_>>())
    }

    fn to_json_required(&self) -> Value {
        json!(self.base.is_required)
    }
    fn to_json_attributes(&self) -> Value {
        json!(self.base.html_attributes)
    }

    fn to_json_meta(&self) -> Value {
        let mut meta = serde_json::Map::new();
        meta.insert(
            "allowed_extensions".to_string(),
            json!(self.allowed_extensions.extensions),
        );
        if let Some(size) = self.upload_config.max_size_mb {
            meta.insert("max_size_mb".to_string(), json!(size));
        }
        if let Some(count) = self.max_files {
            meta.insert("max_files".to_string(), json!(count));
            meta.insert("multiple".to_string(), json!(count > 1));
        }
        if let Some(w) = self.max_width {
            meta.insert("max_width".to_string(), json!(w));
        }
        if let Some(h) = self.max_height {
            meta.insert("max_height".to_string(), json!(h));
        }
        Value::Object(meta)
    }
    fn name(&self) -> &str {
        &self.base.name
    }
    fn label(&self) -> &str {
        &self.base.label
    }
    fn value(&self) -> &str {
        &self.base.value
    }
    fn placeholder(&self) -> &str {
        &self.base.placeholder
    }
    fn field_type(&self) -> &str {
        &self.base.type_field
    }
    fn required(&self) -> bool {
        self.base.is_required.choice
    }
    fn error(&self) -> Option<&String> {
        self.base.error.as_ref()
    }
    fn set_name(&mut self, name: &str) {
        self.base.name = name.to_string();
    }
    fn set_label(&mut self, label: &str) {
        self.base.label = label.to_string();
    }
    fn set_value(&mut self, value: &str) {
        self.base.value = value.to_string();
    }
    fn set_placeholder(&mut self, p: &str) {
        self.base.placeholder = p.to_string();
    }
    fn set_error(&mut self, message: String) {
        self.base.error = if message.is_empty() {
            None
        } else {
            Some(message)
        };
    }
    fn set_required(&mut self, required: bool, msg: Option<&str>) {
        self.base.is_required = BoolChoice {
            choice: required,
            message: msg.map(|s| s.to_string()),
        };
    }
    fn set_html_attribute(&mut self, key: &str, value: &str) {
        self.base
            .html_attributes
            .insert(key.to_string(), value.to_string());
    }
}
