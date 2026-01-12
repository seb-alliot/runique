use crate::formulaire::composant_field::utils::upload::UploadConfig;
use crate::formulaire::field::RuniqueField;
use serde_json::Value;

pub struct MultipleFileField {
    pub accept: String,
    pub min_files: Option<usize>,
    pub max_files: Option<usize>,
    pub max_size_mb: Option<f64>,
    pub upload_to: UploadConfig,
}

impl MultipleFileField {
    pub fn new(accept: &str) -> Self {
        Self {
            accept: accept.to_string(),
            min_files: Some(2),
            max_files: None,
            max_size_mb: None,
            upload_to: UploadConfig::new(),
        }
    }

    pub fn with_limits(
        accept: &str,
        min: Option<usize>,
        max: Option<usize>,
        size: Option<f64>,
    ) -> Result<Self, &'static str> {
        if let Some(min) = min {
            if min < 2 {
                return Err("min_files doit être au moins 2 (sinon utilisez FileField)");
            }
        }

        if let (Some(min), Some(max)) = (min, max) {
            if min > max {
                return Err("min_files ne peut pas être supérieur à max_files");
            }
        }

        Ok(Self {
            accept: accept.to_string(),
            min_files: min,
            max_files: max,
            max_size_mb: size,
            upload_to: UploadConfig::new(),
        })
    }

    pub fn max_files(mut self, max: usize) -> Self {
        self.max_files = Some(max);
        self
    }

    pub fn max_size(mut self, size: f64) -> Self {
        self.max_size_mb = Some(size);
        self
    }

    pub fn unlimited_max_files(mut self) -> Self {
        self.max_files = None;
        self
    }

    pub fn unlimited_size(mut self) -> Self {
        self.max_size_mb = None;
        self
    }

    pub fn upload_to(mut self, path: &str) -> Self {
        self.upload_to = UploadConfig::with_path(path);
        self
    }
}

impl RuniqueField for MultipleFileField {
    type Output = Vec<String>;

    fn process(&self, raw_value: &str) -> Result<Self::Output, String> {
        let files: Vec<String> = raw_value
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        if let Some(min) = self.min_files {
            if files.len() < min {
                return Err(format!("Minimum {} fichiers requis.", min));
            }
        }

        if let Some(max) = self.max_files {
            if files.len() > max {
                return Err(format!("Maximum {} fichiers autorisés.", max));
            }
        }

        if files.is_empty() {
            return Err("Aucun fichier sélectionné.".to_string());
        }

        Ok(files)
    }

    fn template_name(&self) -> &str {
        "file-multiple"
    }

    fn get_context(&self) -> Value {
        serde_json::json!({
            "type": "file",
            "multiple": true,
            "accept": self.accept,
            "max_files": self.max_files,
            "min_files": self.min_files
        })
    }
}
