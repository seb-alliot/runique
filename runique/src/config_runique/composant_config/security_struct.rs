use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]

pub struct SecurityConfig {
    pub sanitize_inputs: bool,
    pub strict_csp: bool,
    pub rate_limiting: bool,
    pub enforce_https: bool,
    pub allowed_hosts: Vec<String>,
}


impl SecurityConfig {
    pub fn from_env() -> Self {
        let sanitize_inputs = std::env::var("SANITIZE_INPUTS")
            .map(|v| v.parse().unwrap_or(true))
            .unwrap_or(true);
        let strict_csp = std::env::var("STRICT_CSP")
            .map(|v| v.parse().unwrap_or(true))
            .unwrap_or(true);
        let rate_limiting = std::env::var("RATE_LIMITING")
            .map(|v| v.parse().unwrap_or(true))
            .unwrap_or(true);
        let enforce_https = std::env::var("ENFORCE_HTTPS")
            .map(|v| v.parse().unwrap_or(false))
            .unwrap_or(false);
        let allowed_hosts = std::env::var("ALLOWED_HOSTS")
            .map(|v| v.split(',').map(|s| s.trim().to_string()).collect())
            .unwrap_or_else(|_| vec!["localhost".to_string(), "127.0.0.1".to_string()]);
        Self {
            sanitize_inputs,
            strict_csp,
            rate_limiting,
            enforce_https,
            allowed_hosts,
        }
    }
}
