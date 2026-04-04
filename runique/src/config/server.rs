//! Configuration du serveur HTTP (adresse, port, clé secrète).
use serde::{Deserialize, Serialize};
use std::env;

const DEFAULT_SECRET_KEY: &str = "default_secret_key";

/// Paramètres de liaison du serveur HTTP et clé secrète HMAC.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ServerConfig {
    /// Adresse IP d'écoute (env: `IP_SERVER`, défaut: `127.0.0.1`).
    pub ip_server: String,
    /// Domaine complet `ip:port` construit automatiquement.
    pub domain_server: String,
    /// Port d'écoute (env: `PORT`, défaut: `3000`).
    pub port: u16,
    /// Clé secrète pour HMAC/CSRF (env: `SECRET_KEY`). Un warning est émis si absente.
    pub secret_key: String,
}

impl ServerConfig {
    /// Charge la configuration depuis les variables d'environnement.
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
                        "[runique] WARNING: SECRET_KEY non définie — la clé par défaut est utilisée. \
                        Les tokens CSRF ne sont pas sécurisés. Définissez SECRET_KEY dans votre .env."
                    );
                }
                key
            },
        }
    }
}
