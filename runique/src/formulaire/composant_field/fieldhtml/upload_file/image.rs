use crate::formulaire::composant_field::utils::upload::UploadConfig;
use crate::formulaire::field::RuniqueField;
use serde_json::Value;

pub struct ImageField {
    pub max_size_mb: f64,
    pub upload_to: UploadConfig,
}

impl ImageField {
    pub fn new() -> Self {
        Self {
            max_size_mb: 5.0,
            upload_to: UploadConfig::new(),
        }
    }

    pub fn with_max_size(max_size_mb: f64) -> Self {
        Self {
            max_size_mb,
            upload_to: UploadConfig::new(),
        }
    }

    pub fn upload_to<S: Into<String>>(mut self, path: S) -> Self {
        self.upload_to.set_path(path);
        self
    }

    pub fn max_size(mut self, size_mb: f64) -> Self {
        self.max_size_mb = size_mb;
        self
    }
}

impl Default for ImageField {
    fn default() -> Self {
        Self::new()
    }
}

impl RuniqueField for ImageField {
    type Output = String;

    fn process(&self, raw_value: &str) -> Result<Self::Output, String> {
        if raw_value.is_empty() {
            return Err("Aucun fichier sélectionné.".to_string());
        }

        let lower = raw_value.to_lowercase();
        let valid_extensions = [
            ".jpg", ".jpeg", ".jpe", ".jif", ".jfif", ".pjpeg", ".pjp", ".png", ".apng", ".gif",
            ".webp", ".svg", ".avif", ".bmp", ".ico", ".tiff", ".tif", ".heic", ".heif",
        ];

        if !valid_extensions.iter().any(|ext| lower.ends_with(ext)) {
            return Err("Format d'image non supporté.".to_string());
        }

        Ok(raw_value.to_string())
    }

    fn template_name(&self) -> &str {
        "file"
    }

    fn get_context(&self) -> Value {
        serde_json::json!({
            "type": "file",
            "accept": "image/*",
            "max_size_mb": self.max_size_mb
        })
    }
}
