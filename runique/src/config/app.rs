//! Main Runique application configuration.
use crate::config::{security::SecurityConfig, server::ServerConfig, static_files::StaticConfig};
use crate::middleware::MiddlewareConfig;
use crate::utils::password::PasswordConfig;
use crate::utils::runique_log::RuniqueLog;
use serde::{Deserialize, Serialize};

/// Aggregated global configuration: server, middleware, security, passwords, static files.
/// Built via [`RuniqueConfig::from_env`] which reads environment variables.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RuniqueConfig {
    pub server: ServerConfig,
    pub middleware: MiddlewareConfig,
    pub security: SecurityConfig,
    pub password: PasswordConfig,
    pub static_files: StaticConfig,
    /// Log configuration by category — initialized via `.with_log()`.
    #[serde(skip)]
    pub log: RuniqueLog,
    pub base_dir: String,
    pub debug: bool,
    /// IANA timezone name — read from `TZ` env var. Default: `"UTC"`.
    /// Parse with `chrono-tz`: `config.timezone.parse::<chrono_tz::Tz>()`.
    pub timezone: String,
    /// The admin blocks the developer replaced, served in place of Runique's
    /// whenever Tera loads the templates, the debug reload included. Set by
    /// the builder, never by hand.
    #[doc(hidden)]
    #[serde(skip)]
    pub block_dispatch: Vec<crate::app::templates::BlockDispatch>,
}

impl RuniqueConfig {
    /// Loads configuration from environment variables (reads `.env` via `dotenvy`).
    pub fn from_env() -> Self {
        dotenvy::dotenv().ok();

        Self {
            server: ServerConfig::from_env(),
            middleware: MiddlewareConfig::from_env(),
            security: SecurityConfig::from_env(),
            password: PasswordConfig::auto(),
            static_files: StaticConfig::from_env(),
            base_dir: std::env::var("BASE_DIR").unwrap_or_else(|_| ".".to_string()),
            debug: crate::utils::config::env::debug_from(std::env::var("DEBUG").ok().as_deref()),
            timezone: std::env::var("TZ").unwrap_or_else(|_| "UTC".to_string()),
            block_dispatch: Vec::new(),
            log: RuniqueLog::default(),
        }
    }
}
