use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StaticConfig {
    // Runique internal
    pub base_dir: String,
    pub static_runique_path: String,
    pub static_runique_url: String,
    pub media_runique_path: String,
    pub media_runique_url: String,
    pub templates_runique: String,

    // User project
    pub templates_dir: Vec<String>,
    pub staticfiles_dirs: String,
    pub media_root: String,
    pub static_url: String,
    pub media_url: String,
    pub staticfiles_storage: String,
}

impl Default for StaticConfig {
    fn default() -> Self {
        let base_dir = ".".to_string();
        let runique_root = env!("CARGO_MANIFEST_DIR");

        let static_runique_path = format!("{}/static", runique_root);
        let media_runique_path = format!("{}/media", runique_root);
        let templates_runique = format!("{}/templates", runique_root);
        Self {
            base_dir,
            static_runique_path: static_runique_path,
            static_runique_url: "/runique/static".to_string(),
            media_runique_path: media_runique_path,
            media_runique_url: "/runique/media".to_string(),
            templates_runique: templates_runique,
            templates_dir: vec!["templates".to_string()],
            staticfiles_dirs: "static".to_string(),
            media_root: "media".to_string(),
            static_url: "/static".to_string(),
            media_url: "/media".to_string(),
            staticfiles_storage: "default_storage".to_string(),
        }
    }
}

impl StaticConfig {
    pub fn from_env() -> Self {
        let base_dir = std::env::var("BASE_DIR").unwrap_or_else(|_| ".".to_string());
        let static_runique_path = std::env::var("STATIC_RUNIQUE_PATH")
            .unwrap_or_else(|_| format!("{}/static", env!("CARGO_MANIFEST_DIR")));
        let static_runique_url =
            std::env::var("STATIC_RUNIQUE_URL").unwrap_or_else(|_| "/runique/static".to_string());
        let media_runique_path = std::env::var("MEDIA_RUNIQUE_PATH")
            .unwrap_or_else(|_| format!("{}/media", env!("CARGO_MANIFEST_DIR")));
        let media_runique_url =
            std::env::var("MEDIA_RUNIQUE_URL").unwrap_or_else(|_| "/runique/media".to_string());
        let templates_runique = std::env::var("TEMPLATES_RUNIQUE")
            .unwrap_or_else(|_| format!("{}/templates", env!("CARGO_MANIFEST_DIR")));
        let templates_dir = std::env::var("TEMPLATES_DIR")
            .map(|v| v.split(',').map(|s| s.trim().to_string()).collect())
            .unwrap_or_else(|_| vec!["templates".to_string()]);
        let staticfiles_dirs =
            std::env::var("STATICFILES_DIRS").unwrap_or_else(|_| "static".to_string());
        let media_root = std::env::var("MEDIA_ROOT").unwrap_or_else(|_| "media".to_string());
        let static_url = std::env::var("STATIC_URL").unwrap_or_else(|_| "/static".to_string());
        let media_url = std::env::var("MEDIA_URL").unwrap_or_else(|_| "/media".to_string());
        let staticfiles_storage =
            std::env::var("STATICFILES_STORAGE").unwrap_or_else(|_| "default_storage".to_string());
        Self {
            base_dir,
            static_runique_path,
            static_runique_url,
            media_runique_path,
            media_runique_url,
            templates_runique,
            templates_dir,
            staticfiles_dirs,
            media_root,
            static_url,
            media_url,
            staticfiles_storage,
        }
    }
}
