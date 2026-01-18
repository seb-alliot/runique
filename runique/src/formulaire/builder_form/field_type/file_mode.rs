use crate::formulaire::builder_form::base_struct::FieldConfig;
use crate::formulaire::builder_form::option_field::BoolChoice;
use crate::formulaire::builder_form::trait_form::FormField;
use serde::Serialize;
use serde_json::{json, Value};
use std::sync::Arc;
use tera::{Context, Tera};

/// Extensions de fichiers autorisées
#[derive(Clone, Debug, Serialize)]
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
        Self::new(vec!["jpg", "jpeg", "png", "gif", "webp", "svg"])
    }

    pub fn documents() -> Self {
        Self::new(vec!["pdf", "doc", "docx", "txt", "odt"])
    }

    pub fn any() -> Self {
        Self { extensions: vec![] }
    }

    pub fn is_allowed(&self, filename: &str) -> bool {
        if self.extensions.is_empty() {
            return true;
        }

        let ext = std::path::Path::new(filename)
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("");

        self.extensions
            .iter()
            .any(|allowed| allowed.eq_ignore_ascii_case(ext))
    }
}

/// FileField - Upload de fichier générique
#[derive(Clone, Serialize, Debug)]
pub struct FileField {
    pub base: FieldConfig,
    pub allowed_extensions: AllowedExtensions,
    pub max_size_mb: Option<f64>,
}

impl FileField {
    pub fn new(name: &str) -> Self {
        Self {
            base: FieldConfig::new(name, "file", "base_file"),
            allowed_extensions: AllowedExtensions::any(),
            max_size_mb: None,
        }
    }

    pub fn allowed_extensions(mut self, extensions: Vec<&str>) -> Self {
        self.allowed_extensions = AllowedExtensions::new(extensions);
        self
    }

    pub fn max_size_mb(mut self, size: f64) -> Self {
        self.max_size_mb = Some(size);
        self
    }

    pub fn label(mut self, label: &str) -> Self {
        self.base.label = label.to_string();
        self
    }

    pub fn required(mut self, msg: &str) -> Self {
        self.set_required(true, Some(msg));
        self
    }
}

impl FormField for FileField {
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

    fn is_required(&self) -> bool {
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
        // La valeur est le nom du fichier uploadé (UUID généré par parse_multipart)
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

    fn validate(&mut self) -> bool {
        let val = self.base.value.trim();

        if self.base.is_required.choice && val.is_empty() {
            let msg = self
                .base
                .is_required
                .message
                .clone()
                .unwrap_or_else(|| "Veuillez sélectionner un fichier".into());
            self.set_error(msg);
            return false;
        }

        if !val.is_empty() {
            // Vérifier l'extension
            if !self.allowed_extensions.is_allowed(val) {
                let exts = self.allowed_extensions.extensions.join(", ");
                self.set_error(format!("Extensions autorisées : {}", exts));
                return false;
            }
        }

        self.set_error("".into());
        true
    }

    fn render(&self, tera: &Arc<Tera>) -> Result<String, String> {
        let mut context = Context::new();
        context.insert("field", &self.base);
        context.insert("allowed_extensions", &self.allowed_extensions);

        if let Some(size) = self.max_size_mb {
            context.insert("max_size_mb", &size);
        }

        tera.render(&self.base.template_name, &context)
            .map_err(|e| e.to_string())
    }

    fn to_json_value(&self) -> Value {
        json!(self.base.value)
    }

    fn to_json_required(&self) -> Value {
        json!(self.base.is_required)
    }

    fn to_json_attributes(&self) -> Value {
        json!(self.base.html_attributes)
    }
}

/// ImageField - Upload d'image avec validation MIME
#[derive(Clone, Serialize, Debug)]
pub struct ImageField {
    pub base: FieldConfig,
    pub max_size_mb: Option<f64>,
    pub max_width: Option<u32>,
    pub max_height: Option<u32>,
}

impl ImageField {
    pub fn new(name: &str) -> Self {
        let mut field = Self {
            base: FieldConfig::new(name, "file", "base_file"),
            max_size_mb: None,
            max_width: None,
            max_height: None,
        };

        // Ajouter l'attribut accept pour images uniquement
        field
            .base
            .html_attributes
            .insert("accept".to_string(), "image/*".to_string());

        field
    }

    pub fn max_size_mb(mut self, size: f64) -> Self {
        self.max_size_mb = Some(size);
        self
    }

    pub fn max_dimensions(mut self, width: u32, height: u32) -> Self {
        self.max_width = Some(width);
        self.max_height = Some(height);
        self
    }

    pub fn label(mut self, label: &str) -> Self {
        self.base.label = label.to_string();
        self
    }

    pub fn required(mut self, msg: &str) -> Self {
        self.set_required(true, Some(msg));
        self
    }
}

impl FormField for ImageField {
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

    fn is_required(&self) -> bool {
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

    fn validate(&mut self) -> bool {
        let val = self.base.value.trim();

        if self.base.is_required.choice && val.is_empty() {
            let msg = self
                .base
                .is_required
                .message
                .clone()
                .unwrap_or_else(|| "Veuillez sélectionner une image".into());
            self.set_error(msg);
            return false;
        }

        if !val.is_empty() {
            // Vérifier que c'est une image (par extension)
            let allowed = AllowedExtensions::images();
            if !allowed.is_allowed(val) {
                self.set_error("Le fichier doit être une image (jpg, png, gif, webp, svg)".into());
                return false;
            }
        }

        self.set_error("".into());
        true
    }

    fn render(&self, tera: &Arc<Tera>) -> Result<String, String> {
        let mut context = Context::new();
        context.insert("field", &self.base);
        context.insert("is_image", &true);

        if let Some(size) = self.max_size_mb {
            context.insert("max_size_mb", &size);
        }

        if let Some(width) = self.max_width {
            context.insert("max_width", &width);
        }

        if let Some(height) = self.max_height {
            context.insert("max_height", &height);
        }

        tera.render(&self.base.template_name, &context)
            .map_err(|e| e.to_string())
    }

    fn to_json_value(&self) -> Value {
        json!(self.base.value)
    }

    fn to_json_required(&self) -> Value {
        json!(self.base.is_required)
    }

    fn to_json_attributes(&self) -> Value {
        json!(self.base.html_attributes)
    }
}

/// MultipleFileField - Upload multiple de fichiers
#[derive(Clone, Serialize, Debug)]
pub struct MultipleFileField {
    pub base: FieldConfig,
    pub allowed_extensions: AllowedExtensions,
    pub max_size_mb: Option<f64>,
    pub max_files: Option<usize>,
}

impl MultipleFileField {
    pub fn new(name: &str) -> Self {
        let mut field = Self {
            base: FieldConfig::new(name, "file", "base_file"),
            allowed_extensions: AllowedExtensions::any(),
            max_size_mb: None,
            max_files: None,
        };

        // Ajouter l'attribut multiple
        field
            .base
            .html_attributes
            .insert("multiple".to_string(), "multiple".to_string());

        field
    }

    pub fn allowed_extensions(mut self, extensions: Vec<&str>) -> Self {
        self.allowed_extensions = AllowedExtensions::new(extensions);
        self
    }

    pub fn max_size_mb(mut self, size: f64) -> Self {
        self.max_size_mb = Some(size);
        self
    }

    pub fn max_files(mut self, count: usize) -> Self {
        self.max_files = Some(count);
        self
    }

    pub fn label(mut self, label: &str) -> Self {
        self.base.label = label.to_string();
        self
    }

    pub fn required(mut self, msg: &str) -> Self {
        self.set_required(true, Some(msg));
        self
    }
}

impl FormField for MultipleFileField {
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

    fn is_required(&self) -> bool {
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
        // La valeur est une liste de noms de fichiers séparés par des virgules
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

    fn validate(&mut self) -> bool {
        let val = self.base.value.trim();

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
            // Séparer les noms de fichiers (format: "file1.jpg,file2.png")
            let files: Vec<&str> = val.split(',').map(|s| s.trim()).collect();

            // Vérifier le nombre max de fichiers
            if let Some(max) = self.max_files {
                if files.len() > max {
                    self.set_error(format!("Maximum {} fichiers autorisés", max));
                    return false;
                }
            }

            // Vérifier les extensions
            for filename in files {
                if !self.allowed_extensions.is_allowed(filename) {
                    let exts = self.allowed_extensions.extensions.join(", ");
                    self.set_error(format!(
                        "Fichier '{}' : extensions autorisées : {}",
                        filename, exts
                    ));
                    return false;
                }
            }
        }

        self.set_error("".into());
        true
    }

    fn render(&self, tera: &Arc<Tera>) -> Result<String, String> {
        let mut context = Context::new();
        context.insert("field", &self.base);
        context.insert("allowed_extensions", &self.allowed_extensions);
        context.insert("multiple", &true);

        if let Some(size) = self.max_size_mb {
            context.insert("max_size_mb", &size);
        }

        if let Some(count) = self.max_files {
            context.insert("max_files", &count);
        }

        tera.render(&self.base.template_name, &context)
            .map_err(|e| e.to_string())
    }

    fn to_json_value(&self) -> Value {
        // Retourner un array de noms de fichiers
        let files: Vec<&str> = self
            .base
            .value
            .split(',')
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .collect();
        json!(files)
    }

    fn to_json_required(&self) -> Value {
        json!(self.base.is_required)
    }

    fn to_json_attributes(&self) -> Value {
        json!(self.base.html_attributes)
    }
}
