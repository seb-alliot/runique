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
        Self {
            strict_csp,
            rate_limiting,
            enforce_https,
            allowed_hosts,
        }
    }
}
