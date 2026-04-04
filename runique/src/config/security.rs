//! Paramètres de sécurité globaux (CSP, rate limiting, HTTPS, hôtes autorisés).
use serde::{Deserialize, Serialize};

/// Paramètres de sécurité lus depuis l'environnement.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SecurityConfig {
    /// Active la Content Security Policy stricte (env: `STRICT_CSP`, défaut: `true`).
    pub strict_csp: bool,
    /// Active le rate limiting global (env: `RATE_LIMITING`, défaut: `true`).
    pub rate_limiting: bool,
    /// Redirige HTTP vers HTTPS (env: `ENFORCE_HTTPS`, défaut: `false`).
    pub enforce_https: bool,
    /// Liste des hôtes autorisés (env: `ALLOWED_HOSTS`, séparés par virgule).
    pub allowed_hosts: Vec<String>,
}

impl SecurityConfig {
    /// Charge la configuration depuis les variables d'environnement.
    pub fn from_env() -> Self {
        let strict_csp = std::env::var("STRICT_CSP")
            .map(|v| v.parse().unwrap_or(true))
            .unwrap_or(true);
        let rate_limiting = std::env::var("RATE_LIMITING")
            .map(|v| v.parse().unwrap_or(true))
            .unwrap_or(true);
        let enforce_https = std::env::var("ENFORCE_HTTPS")
            .map(|v| v.parse().unwrap_or(false))
            .unwrap_or(false);
        let allowed_hosts: Vec<String> = std::env::var("ALLOWED_HOSTS")
            .map(|v| v.split(',').map(|s| s.trim().to_string()).collect())
            .unwrap_or_else(|_| vec!["localhost".to_string(), "127.0.0.1".to_string()]);
        Self {
            strict_csp,
            rate_limiting,
            enforce_https,
            allowed_hosts,
        }
    }
}
