//! HTTP server configuration (address, port, secret key).
use serde::{Deserialize, Serialize};
use std::env;

const DEFAULT_SECRET_KEY: &str = "default_secret_key";

/// Longueur minimale d'une `SECRET_KEY` exploitable (HMAC-SHA256 = 256 bits = 32 octets).
pub const MIN_SECRET_KEY_LEN: usize = 32;

/// Une clé est **inexploitable** pour le HMAC/CSRF si elle est vide, vaut la valeur par
/// défaut, ou fait moins de [`MIN_SECRET_KEY_LEN`] caractères. La boot validation s'en sert
/// pour refuser le démarrage en production (en debug : simple warning).
#[must_use]
pub fn secret_key_is_weak(key: &str) -> bool {
    let k = key.trim();
    k.is_empty() || k == DEFAULT_SECRET_KEY || k.chars().count() < MIN_SECRET_KEY_LEN
}

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

#[cfg(test)]
mod secret_key_tests {
    use super::{MIN_SECRET_KEY_LEN, secret_key_is_weak};

    #[test]
    fn weak_keys_are_rejected() {
        assert!(secret_key_is_weak(""), "vide");
        assert!(secret_key_is_weak("   "), "blancs uniquement");
        assert!(secret_key_is_weak("default_secret_key"), "clé par défaut");
        assert!(secret_key_is_weak("short"), "trop courte");
        assert!(
            secret_key_is_weak(&"a".repeat(MIN_SECRET_KEY_LEN - 1)),
            "31 < 32"
        );
    }

    #[test]
    fn strong_key_is_accepted() {
        assert!(
            !secret_key_is_weak(&"a".repeat(MIN_SECRET_KEY_LEN)),
            "32 OK"
        );
        assert!(!secret_key_is_weak("kZ9!xQ2_7pLmW4vR8nB1tY6cD3eF0sHj")); // 32 chars
    }
}
