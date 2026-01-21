use crate::config_runique::composant_config::{
    security_struct::SecurityConfig, server_struct::ServerConfig, settings_struct::AppSettings,
    static_struct::StaticConfig,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuniqueConfig {
    pub server: ServerConfig,
    pub security: SecurityConfig,
    pub static_files: StaticConfig,
    pub app: AppSettings,
    pub base_dir: String,
    pub debug: bool,
}

impl RuniqueConfig {
    pub fn from_env() -> Self {
        dotenvy::dotenv().ok();

        // 2. Assembler les composants en essayant de lire l'environnement
        Self {
            server: ServerConfig::from_env(),
            security: SecurityConfig::from_env(),
            static_files: StaticConfig::from_env(),
            app: AppSettings::from_env(),

            base_dir: std::env::var("BASE_DIR").unwrap_or_else(|_| ".".to_string()),
            debug: std::env::var("DEBUG")
                .map(|v| v.parse().unwrap_or(cfg!(debug_assertions)))
                .unwrap_or(cfg!(debug_assertions)),
        }
    }
}
