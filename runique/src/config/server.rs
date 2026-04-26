//! HTTP server configuration (address, port, secret key).
use serde::{Deserialize, Serialize};
use std::env;

const DEFAULT_SECRET_KEY: &str = "default_secret_key";

/// HTTP server binding parameters and HMAC secret key.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ServerConfig {
    /// Listening IP address (env: `IP_SERVER`, default: `127.0.0.1`).
    pub ip_server: String,
    /// Full `ip:port` domain built automatically.
    pub domain_server: String,
    /// Listening port (env: `PORT`, default: `3000`).
    pub port: u16,
    /// Secret key for HMAC/CSRF (env: `SECRET_KEY`). A warning is issued if missing.
    pub secret_key: String,
}

impl ServerConfig {
    /// Loads configuration from environment variables.
    pub fn from_env() -> Self {
        let ip = env::var("IP_SERVER").unwrap_or_else(|_| "127.0.0.1".to_string());
        let port: u16 = env::var("PORT")
            .ok()
            .and_then(|p| p.parse().ok())
            .unwrap_or(3000);

        ServerConfig {
            ip_server: ip.clone(),
            domain_server: format!("{}:{}", ip, port),
            port,
            secret_key: {
                let key = env::var("SECRET_KEY").unwrap_or_else(|_| DEFAULT_SECRET_KEY.to_string());
                if key == DEFAULT_SECRET_KEY {
                    eprintln!(
                        "[runique] WARNING: SECRET_KEY is not defined — using default key. \
                        CSRF tokens are not secure. Define SECRET_KEY in your .env file."
                    );
                }
                key
            },
        }
    }
}
