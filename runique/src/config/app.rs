//! Configuration principale de l'application Runique.
use crate::config::{security::SecurityConfig, server::ServerConfig, static_files::StaticConfig};
use crate::middleware::MiddlewareConfig;
use crate::utils::password::PasswordConfig;
use crate::utils::runique_log::RuniqueLog;
use serde::{Deserialize, Serialize};

/// Configuration globale agrégée : serveur, middleware, sécurité, mots de passe, fichiers statiques.
/// Construite via [`RuniqueConfig::from_env`] qui lit les variables d'environnement.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RuniqueConfig {
    pub server: ServerConfig,
    pub middleware: MiddlewareConfig,
    pub security: SecurityConfig,
    pub password: PasswordConfig,
    pub static_files: StaticConfig,
    /// Configuration des logs par catégorie — initialisée via `.with_log()`.
    #[serde(skip)]
    pub log: RuniqueLog,
    pub base_dir: String,
    pub debug: bool,
}

impl RuniqueConfig {
    /// Charge la configuration depuis les variables d'environnement (lit `.env` via dotenvy).
    pub fn from_env() -> Self {
        dotenvy::dotenv().ok();

        Self {
            server: ServerConfig::from_env(),
            middleware: MiddlewareConfig::from_env(),
            security: SecurityConfig::from_env(),
            password: PasswordConfig::auto(),
            static_files: StaticConfig::from_env(),
            base_dir: std::env::var("BASE_DIR").unwrap_or_else(|_| ".".to_string()),
            debug: matches!(std::env::var("DEBUG").as_deref(), Ok("true" | "1")),
            log: RuniqueLog::default(),
        }
    }
}
