//! Global security settings (CSP, rate limiting, HTTPS, allowed hosts).
use serde::{Deserialize, Serialize};

/// Security settings read from the environment.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SecurityConfig {
    /// Enables strict Content Security Policy (env: `STRICT_CSP`, default: `true`).
    pub strict_csp: bool,
    /// Enables global rate limiting (env: `RATE_LIMITING`, default: `true`).
    pub rate_limiting: bool,
    /// Redirects HTTP to HTTPS (env: `ENFORCE_HTTPS`, default: `false`).
    pub enforce_https: bool,
    /// List of allowed hosts (env: `ALLOWED_HOSTS`, comma-separated).
    pub allowed_hosts: Vec<String>,
    /// Enables automatic TLS via Let's Encrypt ACME (env: `ACME_ENABLED`, default: `false`).
    pub acme_enabled: bool,
    /// Domain for ACME certificate (env: `ACME_DOMAIN`).
    pub acme_domain: Option<String>,
    /// Contact email for Let's Encrypt account (env: `ACME_EMAIL`).
    pub acme_email: Option<String>,
    /// Directory where TLS certificates are stored (env: `ACME_CERTS_DIR`, default: `./certs`).
    pub acme_certs_dir: String,
}

impl SecurityConfig {
    /// Loads configuration from environment variables.
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
        let acme_enabled = std::env::var("ACME_ENABLED")
            .map(|v| v.parse().unwrap_or(false))
            .unwrap_or(false);
        let acme_domain = std::env::var("ACME_DOMAIN").ok().filter(|s| !s.is_empty());
        let acme_email = std::env::var("ACME_EMAIL").ok().filter(|s| !s.is_empty());
        let acme_certs_dir = std::env::var("ACME_CERTS_DIR")
            .ok()
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| "./certs".to_string());

        Self {
            strict_csp,
            rate_limiting,
            enforce_https,
            allowed_hosts,
            acme_enabled,
            acme_domain,
            acme_email,
            acme_certs_dir,
        }
    }

    /// HSTS ne doit être émis QUE si Runique sert réellement du HTTPS : terminaison TLS
    /// via ACME, ou redirection HTTPS forcée. En HTTP simple le header serait ignoré, et
    /// surtout on évite le lock-in HTTPS d'un an (`max-age` + `includeSubDomains`/`preload`)
    /// sur un déploiement qui n'est pas (encore) en HTTPS.
    #[must_use]
    pub fn should_emit_hsts(&self) -> bool {
        self.enforce_https || self.acme_enabled
    }
}

#[cfg(test)]
mod hsts_tests {
    use super::*;

    fn cfg(enforce_https: bool, acme: bool) -> SecurityConfig {
        SecurityConfig {
            strict_csp: true,
            rate_limiting: true,
            enforce_https,
            allowed_hosts: vec![],
            acme_enabled: acme,
            acme_domain: None,
            acme_email: None,
            acme_certs_dir: String::new(),
        }
    }

    /// HSTS gaté sur HTTPS réel : pas de lock-in HTTPS forcé en HTTP simple.
    #[test]
    fn hsts_only_over_real_https() {
        assert!(
            !cfg(false, false).should_emit_hsts(),
            "HTTP simple → pas de HSTS"
        );
        assert!(cfg(true, false).should_emit_hsts(), "enforce_https → HSTS");
        assert!(
            cfg(false, true).should_emit_hsts(),
            "ACME (Runique sert le TLS) → HSTS"
        );
    }
}
