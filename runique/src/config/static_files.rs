use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct StaticConfig {
    // Runique internal
    pub base_dir: String,
    pub static_runique_path: String,
    pub static_runique_url: String,
    pub media_runique_path: String,
    pub media_runique: String,
    pub templates_runique: String,

    // User project
    pub templates_dir: Vec<String>,
    pub staticfiles_dirs: String,
    pub media_root: String,
    pub static_url: String,
    pub media_url: String,
    pub staticfiles: String,
    pub og_image: String,
    /// Taille maximale d'un fichier uploadé en Mo (env: RUNIQUE_MAX_UPLOAD_MB, défaut: 100).
    /// Protection DoS au niveau du streaming — la limite par champ (`FileField::max_size_mb`)
    /// reste le contrôle fonctionnel.
    pub max_upload_mb: u64,
    /// Taille maximale d'un champ texte multipart en Ko (env: RUNIQUE_MAX_TEXT_FIELD_KB, défaut: 1024).
    pub max_text_field_kb: usize,
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

        let media_runique =
            std::env::var("MEDIA_RUNIQUE_URL").unwrap_or_else(|_| "/runique/media".to_string());

        let og_image =
            std::env::var("OG_IMAGE").unwrap_or("/runique/static/favicon_runique.ico".to_string());

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

        let staticfiles =
            std::env::var("STATICFILES").unwrap_or_else(|_| "default_storage".to_string());

        let max_upload_mb = std::env::var("RUNIQUE_MAX_UPLOAD_MB")
            .ok()
            .and_then(|v| v.parse::<u64>().ok())
            .unwrap_or(100);

        let max_text_field_kb = std::env::var("RUNIQUE_MAX_TEXT_FIELD_KB")
            .ok()
            .and_then(|v| v.parse::<usize>().ok())
            .unwrap_or(1024);

        Self {
            base_dir,
            static_runique_path,
            static_runique_url,
            media_runique_path,
            media_runique,
            og_image,
            templates_runique,
            templates_dir,
            staticfiles_dirs,
            media_root,
            static_url,
            media_url,
            staticfiles,
            max_upload_mb,
            max_text_field_kb,
        }
    }
}
