#[derive(Debug, Clone)]
pub struct UploadConfig {
    pub path: Option<String>,
}

impl UploadConfig {
    pub fn new() -> Self {
        Self { path: None }
    }

    pub fn with_path<S: Into<String>>(path: S) -> Self {
        Self {
            path: Some(path.into()),
        }
    }

    pub fn set_path<S: Into<String>>(&mut self, path: S) {
        self.path = Some(path.into());
    }

    pub fn get_path(&self) -> String {
        self.path.clone().unwrap_or_else(|| "uploads/".to_string())
    }
}

impl Default for UploadConfig {
    fn default() -> Self {
        Self::new()
    }
}
