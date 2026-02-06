// runique/src/middleware/allowed_hosts.rs

use crate::utils::aliases::AEngine;
use axum::{
    body::Body,
    extract::State,
    http::{header, HeaderMap, Request, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HostPolicy {
    pub allowed_hosts: Vec<String>,
    pub debug: bool, // Ajouté pour la cohérence avec is_host_allowed
}

impl HostPolicy {
    pub fn new(allowed_hosts: Vec<String>, debug: bool) -> Self {
        Self {
            allowed_hosts,
            debug,
        }
    }

    pub fn from_env() -> Self {
        let hosts = std::env::var("RUNIQUE_POLICY_ALLOWED_HOSTS")
            .unwrap_or_else(|_| "localhost:3000".to_string())
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s: &String| !s.is_empty())
            .collect();

        // On synchronise le mode debug avec l'interrupteur global
        let debug = std::env::var("RUNIQUE_ENABLE_DEBUG_ERRORS")
            .map(|v| v.parse().unwrap_or(false))
            .unwrap_or(false);

        Self {
            allowed_hosts: hosts,
            debug,
        }
    }

    pub fn is_host_allowed(&self, host: &str) -> bool {
        if self.debug {
            return true;
        } //
        fn normalize_host(host: &str) -> &str {
            if host.starts_with('[') {
                // IPv6, garder jusqu'à ]
                host.split(']')
                    .next()
                    .map(|h| &host[..h.len() + 1])
                    .unwrap_or(host)
            } else {
                host.split(':').next().unwrap_or(host)
            }
        }
        let host = normalize_host(host);

        self.allowed_hosts.iter().any(|allowed| {
            if allowed == "*" {
                true
            } else if allowed.starts_with('.') {
                if host == &allowed[1..] {
                    true
                } else if host.ends_with(allowed) {
                    let match_start = host.len() - allowed.len();
                    match_start > 0 && host.as_bytes()[match_start] == b'.'
                } else {
                    false
                }
            } else {
                allowed == host
            }
        })
    }

    pub fn validate(&self, headers: &HeaderMap) -> Result<(), (StatusCode, String)> {
        let host = match headers.get(header::HOST) {
            Some(h) => h.to_str().unwrap_or("<invalid host header>"),
            None => {
                return Err((
                    StatusCode::BAD_REQUEST,
                    self.make_error_message("<no host>"),
                ))
            }
        };

        if !self.is_host_allowed(host) {
            return Err((StatusCode::BAD_REQUEST, self.make_error_message(host)));
        }
        Ok(())
    }

    fn make_error_message(&self, host: &str) -> String {
        if self.debug {
            format!(
                "Invalid Host: '{}'. Add it to RUNIQUE_POLICY_ALLOWED_HOSTS.",
                host
            )
        } else {
            "Bad Request".to_string()
        }
    }
}

pub async fn allowed_hosts_middleware(
    State(engine): State<AEngine>, // Utilise maintenant ton Engine central
    request: Request<Body>,
    next: Next,
) -> Response {
    // On utilise les "meubles" déjà chargés dans l'engine au démarrage
    if let Err((status, message)) = engine.security_hosts.validate(request.headers()) {
        return (status, message).into_response();
    }

    next.run(request).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exact_match() {
        let validator = HostPolicy::new(vec!["exemple.com".to_string()], false);

        assert!(validator.is_host_allowed("exemple.com"));
        assert!(!validator.is_host_allowed("www.exemple.com"));
        assert!(!validator.is_host_allowed("malicious.com"));
    }

    #[test]
    fn test_wildcard_subdomain() {
        let validator = HostPolicy::new(vec![".exemple.com".to_string()], false);
        assert!(validator.is_host_allowed("exemple.com"));
        assert!(validator.is_host_allowed("www.exemple.com"));
        assert!(validator.is_host_allowed("api.exemple.com"));
        assert!(validator.is_host_allowed("admin.api.exemple.com"));
        assert!(!validator.is_host_allowed("malicious.com"));
    }

    #[test]
    fn test_wildcard_all() {
        let validator = HostPolicy::new(vec!["*".to_string()], false);

        assert!(validator.is_host_allowed("exemple.com"));
        assert!(validator.is_host_allowed("n-importe-quoi.com"));
    }

    #[test]
    fn test_multiple_hosts() {
        let validator = HostPolicy::new(
            vec![
                "exemple.com".to_string(),
                "www.exemple.com".to_string(),
                ".api.exemple.com".to_string(),
            ],
            false,
        );

        assert!(validator.is_host_allowed("exemple.com"));
        assert!(validator.is_host_allowed("www.exemple.com"));
        assert!(validator.is_host_allowed("api.exemple.com"));
        assert!(validator.is_host_allowed("v1.api.exemple.com"));
        assert!(!validator.is_host_allowed("autre.exemple.com"));
    }

    #[test]
    fn test_host_with_port() {
        let validator = HostPolicy::new(vec!["exemple.com".to_string()], false);

        assert!(validator.is_host_allowed("exemple.com:8080"));
        assert!(validator.is_host_allowed("exemple.com:443"));
    }

    #[test]
    fn test_debug_mode_allows_all() {
        let validator = HostPolicy::new(
            vec!["exemple.com".to_string()],
            true, // debug = true
        );

        assert!(validator.is_host_allowed("n-importe-quoi.com"));
        assert!(validator.is_host_allowed("malicious.com"));
    }

    #[test]
    fn test_wildcard_subdomain_security() {
        // Test pour éviter que "malicious-exemple.com" match ".exemple.com"
        let validator = HostPolicy::new(vec![".exemple.com".to_string()], false);

        // Doit matcher
        assert!(validator.is_host_allowed("exemple.com"));
        assert!(validator.is_host_allowed("www.exemple.com"));
        assert!(validator.is_host_allowed("api.exemple.com"));

        // Ne doit PAS matcher (bug de sécurité)
        assert!(!validator.is_host_allowed("malicious-exemple.com"));
        assert!(!validator.is_host_allowed("evil-exemple.com"));
        assert!(!validator.is_host_allowed("exemple.com.evil.com"));
    }
}
