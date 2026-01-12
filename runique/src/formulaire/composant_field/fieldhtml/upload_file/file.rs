use crate::formulaire::field::RuniqueField;
use crate::formulaire::composant_field::utils::upload::UploadConfig;

pub struct FileField {
    pub accept: String,
    pub size: Option<f64>,
    pub upload_to: UploadConfig,
}

impl FileField {
    pub fn new(accept: &str) -> Self {
        Self {
            accept: accept.to_string(),
            size: None,
            upload_to: UploadConfig::new(),
        }
    }

    pub fn with_size(mut self, size_mb: f64) -> Self {
        self.size = Some(size_mb);
        self
    }

    pub fn upload_to(mut self, path: &str) -> Self {
        self.upload_to = UploadConfig::with_path(path);
        self
    }

    pub fn unlimited_size(mut self) -> Self {
        self.size = None;
        self
    }
}

impl RuniqueField for FileField {
    type Output = String;

    fn process(&self, raw_value: &str) -> Result<Self::Output, String> {
        Ok(raw_value.to_string())
    }

    fn template_name(&self) -> &str {
        "file"
    }

    fn get_context(&self) -> serde_json::Value {
        serde_json::json!({ "accept": self.accept, "size": self.size })
    }
}