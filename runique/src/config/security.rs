//! Global security settings (CSP, rate limiting, HTTPS, allowed hosts).
use serde::{Deserialize, Serialize};

/// Security settings read from the environment.
#[derive(Debug, Clone, Serialize, Deserialize)]
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
    /// HSTS `max-age` in seconds (env: `HSTS_MAX_AGE`, default: `31536000` = 1 an).
    pub hsts_max_age: u64,
    /// HSTS `includeSubDomains` (env: `HSTS_INCLUDE_SUBDOMAINS`, default: `true`).
    /// ⚠️ footgun : casse tout sous-domaine qui n'est pas en HTTPS.
    pub hsts_include_subdomains: bool,
    /// HSTS `preload` (env: `HSTS_PRELOAD`, default: `false`). Engagement quasi-
    /// irréversible (soumission à la liste des navigateurs) → opt-in explicite.
    pub hsts_preload: bool,
}

impl Default for SecurityConfig {
    /// Preserves the historical "everything off" default for the pre-existing
    /// fields, but gives HSTS **sane** defaults — a `max-age` of `0` (what a
    /// derived `Default` would produce) actively *disables* HSTS in browsers,
    /// so the default must be the real 1-year value, `includeSubDomains`, no
    /// preload. Emission stays gated by `should_emit_hsts()`.
    fn default() -> Self {
        Self {
            strict_csp: false,
            rate_limiting: false,
            enforce_https: false,
            allowed_hosts: Vec::new(),
            acme_enabled: false,
            acme_domain: None,
            acme_email: None,
            acme_certs_dir: String::new(),
            hsts_max_age: 31_536_000,
            hsts_include_subdomains: true,
            hsts_preload: false,
        }
    }
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
        let hsts_max_age = std::env::var("HSTS_MAX_AGE")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(31_536_000);
        let hsts_include_subdomains = std::env::var("HSTS_INCLUDE_SUBDOMAINS")
            .map(|v| v.parse().unwrap_or(true))
            .unwrap_or(true);
        let hsts_preload = std::env::var("HSTS_PRELOAD")
            .map(|v| v.parse().unwrap_or(false))
            .unwrap_or(false);

        Self {
            strict_csp,
            rate_limiting,
            enforce_https,
            allowed_hosts,
            acme_enabled,
            acme_domain,
            acme_email,
            acme_certs_dir,
            hsts_max_age,
            hsts_include_subdomains,
            hsts_preload,
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

    /// The `Strict-Transport-Security` header value to emit, or `None` when HSTS
    /// must not be sent (not over real HTTPS). Single source of truth used by
    /// every emission point (CSP middleware, error pages, static files) so they
    /// can't drift. Built from the configured `max-age`/`includeSubDomains`/`preload`.
    #[must_use]
    pub fn hsts_header_value(&self) -> Option<String> {
        if !self.should_emit_hsts() {
            return None;
        }
        let mut v = format!("max-age={}", self.hsts_max_age);
        if self.hsts_include_subdomains {
            v.push_str("; includeSubDomains");
        }
        if self.hsts_preload {
            v.push_str("; preload");
        }
        Some(v)
    }

    /// `true` when `preload` is set but the combo is invalid for the browser
    /// preload list (`preload` requires `includeSubDomains` **and** `max-age ≥ 1 an`).
    /// Such a header is ignored for preload → worth a boot warning.
    #[must_use]
    pub fn hsts_preload_misconfigured(&self) -> bool {
        self.hsts_preload && (!self.hsts_include_subdomains || self.hsts_max_age < 31_536_000)
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
            hsts_max_age: 31_536_000,
            hsts_include_subdomains: true,
            hsts_preload: false,
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

    #[test]
    fn hsts_value_none_when_not_https() {
        assert_eq!(cfg(false, false).hsts_header_value(), None);
    }

    #[test]
    fn hsts_value_default_has_no_preload() {
        // Default : max-age 1 an + includeSubDomains, JAMAIS preload par défaut.
        assert_eq!(
            cfg(true, false).hsts_header_value().as_deref(),
            Some("max-age=31536000; includeSubDomains")
        );
    }

    #[test]
    fn hsts_value_honors_overrides() {
        let mut c = cfg(true, false);
        c.hsts_max_age = 15_552_000;
        c.hsts_include_subdomains = false;
        c.hsts_preload = true;
        assert_eq!(
            c.hsts_header_value().as_deref(),
            Some("max-age=15552000; preload")
        );
    }

    #[test]
    fn preload_misconfig_detected() {
        let mut c = cfg(true, false);
        c.hsts_preload = true;
        c.hsts_include_subdomains = false; // preload sans includeSubDomains → invalide
        assert!(c.hsts_preload_misconfigured());
        c.hsts_include_subdomains = true;
        c.hsts_max_age = 31_536_000;
        assert!(!c.hsts_preload_misconfigured());
    }
}
