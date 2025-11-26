//! Application configuration module

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Main settings structure - Django-inspired
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    // Basic settings
    pub base_dir: String,
    pub secret_key: String,
    pub debug: bool,
    pub allowed_hosts: Vec<String>,

    // Server settings
    pub server: ServerSettings,

    // Templates
    pub templates_dir: String,

    // Static files
    pub static_url: String,
    pub static_root: String,

    // Media files
    pub media_url: String,
    pub media_root: String,

    // Database
    #[cfg(feature = "orm")]
    pub database: DatabaseSettings,

    // Localization
    pub language_code: String,
    pub time_zone: String,
    pub use_i18n: bool,
    pub use_tz: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerSettings {
    pub host: String,
    pub port: u16,
}

#[cfg(feature = "orm")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseSettings {
    pub engine: String,
    pub name: String,
    pub user: String,
    pub password: String,
    pub host: String,
    pub port: u16,
    pub url: String,
}

impl Default for Settings {
    fn default() -> Self {
        Self::default_values()
    }
}

impl Settings {
    /// Create default settings
    pub fn default_values() -> Self {
        let base_dir = "src";
        let templates_dir = format!("{}/templates", base_dir);
        let static_root = format!("{}/static", base_dir);
        let media_root = format!("{}/media", base_dir);

        Settings {
            base_dir: base_dir.to_string(),
            secret_key: std::env::var("SECRET_KEY")
                .unwrap_or_else(|_| "dev-secret-key-change-in-production".to_string()),
            debug: std::env::var("DEBUG")
                .map(|v| v == "true" || v == "1")
                .unwrap_or(true),
            allowed_hosts: vec![
                "localhost".to_string(),
                "127.0.0.1".to_string(),
            ],

            server: ServerSettings {
                host: std::env::var("HOST").unwrap_or_else(|_| "127.0.0.1".to_string()),
                port: std::env::var("PORT")
                    .ok()
                    .and_then(|p| p.parse().ok())
                    .unwrap_or(3000),
            },

            templates_dir,
            static_url: "/static/".to_string(),
            static_root,
            media_url: "/media/".to_string(),
            media_root,

            #[cfg(feature = "orm")]
            database: DatabaseSettings::from_env(),

            language_code: "en-us".to_string(),
            time_zone: "UTC".to_string(),
            use_i18n: true,
            use_tz: true,
        }
    }

    /// Load from environment variables
    pub fn from_env() -> Self {
        dotenvy::dotenv().ok();
        Self::default_values()
    }

    /// Get the full server address
    pub fn server_addr(&self) -> String {
        format!("{}:{}", self.server.host, self.server.port)
    }
}

#[cfg(feature = "orm")]
impl DatabaseSettings {
    /// Load database settings from environment
    pub fn from_env() -> Self {
        let engine = std::env::var("DB_ENGINE")
            .unwrap_or_else(|_| "sqlite".to_string());
        
        let url = if engine == "postgres" {
            let user = std::env::var("POSTGRES_USER")
                .unwrap_or_else(|_| "postgres".to_string());
            let password = std::env::var("POSTGRES_PASSWORD")
                .unwrap_or_else(|_| "postgres".to_string());
            let host = std::env::var("POSTGRES_HOST")
                .unwrap_or_else(|_| "localhost".to_string());
            let port = std::env::var("POSTGRES_PORT")
                .unwrap_or_else(|_| "5432".to_string());
            let name = std::env::var("POSTGRES_DB")
                .unwrap_or_else(|_| "mydb".to_string());

            format!("postgres://{}:{}@{}:{}/{}", user, password, host, port, name)
        } else {
            std::env::var("DATABASE_URL")
                .unwrap_or_else(|_| "sqlite://db.sqlite?mode=rwc".to_string())
        };

        Self {
            engine: engine.clone(),
            name: std::env::var("DB_NAME").unwrap_or_else(|_| "mydb".to_string()),
            user: std::env::var("DB_USER").unwrap_or_else(|_| "user".to_string()),
            password: std::env::var("DB_PASSWORD").unwrap_or_default(),
            host: std::env::var("DB_HOST").unwrap_or_else(|_| "localhost".to_string()),
            port: std::env::var("DB_PORT")
                .ok()
                .and_then(|p| p.parse().ok())
                .unwrap_or(5432),
            url,
        }
    }
}
