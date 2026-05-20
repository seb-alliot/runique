//! Static files, media, and templates configuration.
use serde::{Deserialize, Serialize};

/// Paths and URLs for framework and user project assets.
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
    /// Maximum upload size for a file in MB (env: RUNIQUE_MAX_UPLOAD_MB, default: 100).
    /// DoS protection at the streaming level - the per-field limit (`FileField::max_size`)
    /// remains the functional control.
    pub max_upload_mb: u64,
    /// Maximum size of a multipart text field in KB (env: RUNIQUE_MAX_TEXT_FIELD_KB, default: 1024).
    pub max_text_field_kb: usize,
}

/// Returns the current working directory as a string, cross-platform.
/// Falls back to `"."` if the OS call fails (deleted dir, permission error).
fn current_dir_str() -> String {
    std::env::current_dir()
        .ok()
        .and_then(|p| p.to_str().map(str::to_string))
        .unwrap_or_else(|| ".".to_string())
}

/// Priority: `MEDIA_ROOT` → `{BASE_DIR}/media` → `{cwd}/media` → `./media`.
pub fn resolve_media_root() -> String {
    if let Ok(root) = std::env::var("MEDIA_ROOT") {
        return root;
    }
    if let Ok(base) = std::env::var("BASE_DIR") {
        return format!("{}/media", base);
    }
    format!("{}/media", current_dir_str())
}

impl StaticConfig {
    /// Loads paths from environment variables with sensible defaults.
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

        let media_root = resolve_media_root();

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

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    // std::env::set_var is not thread-safe — serialize all env-mutating tests.
    static ENV_LOCK: Mutex<()> = Mutex::new(());

    #[test]
    fn current_dir_str_is_absolute() {
        let dir = current_dir_str();
        assert!(!dir.is_empty());
        assert!(
            std::path::Path::new(&dir).is_absolute(),
            "current_dir_str() returned a relative path: {dir}"
        );
    }

    #[test]
    fn resolve_media_root_explicit_wins() {
        let _lock = ENV_LOCK.lock().unwrap();
        unsafe {
            std::env::set_var("MEDIA_ROOT", "/custom/media");
            std::env::remove_var("BASE_DIR");
        }
        let result = resolve_media_root();
        unsafe { std::env::remove_var("MEDIA_ROOT") };
        assert_eq!(result, "/custom/media");
    }

    #[test]
    fn resolve_media_root_uses_base_dir() {
        let _lock = ENV_LOCK.lock().unwrap();
        unsafe {
            std::env::remove_var("MEDIA_ROOT");
            std::env::set_var("BASE_DIR", "/var/www/app");
        }
        let result = resolve_media_root();
        unsafe { std::env::remove_var("BASE_DIR") };
        assert_eq!(result, "/var/www/app/media");
    }

    #[test]
    fn resolve_media_root_falls_back_to_cwd() {
        let _lock = ENV_LOCK.lock().unwrap();
        unsafe {
            std::env::remove_var("MEDIA_ROOT");
            std::env::remove_var("BASE_DIR");
        }
        let result = resolve_media_root();
        assert_eq!(result, format!("{}/media", current_dir_str()));
        assert!(
            std::path::Path::new(&result).is_absolute(),
            "fallback path should be absolute: {result}"
        );
    }
}
