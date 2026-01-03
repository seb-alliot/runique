// runique/src/middleware/allowed_hosts.rs

use axum::{
    extract::Request,
    http::{HeaderMap, StatusCode, header},
    middleware::Next,
    response::{IntoResponse, Response},
};
use std::sync::Arc;

use crate::Settings;

/// Middleware de validation des hosts autorisés
///
/// Protège contre les attaques de type Host Header Injection
/// Inspiré de Django's ALLOWED_HOSTS
#[derive(Debug, Clone)]
pub struct AllowedHostsValidator {
    allowed_hosts: Vec<String>,
    debug: bool,
}

impl AllowedHostsValidator {
    /// Crée un nouveau validateur de hosts
    pub fn new(allowed_hosts: Vec<String>, debug: bool) -> Self {
        Self {
            allowed_hosts,
            debug,
        }
    }

    /// Crée le validateur depuis les Settings
    pub fn from_settings(settings: &Settings) -> Self {
        Self::new(settings.allowed_hosts.clone(), settings.debug)
    }

    /// Vérifie si un host est autorisé
    ///
    /// Supporte:
    /// - Correspondance exacte: "exemple.com"
    /// - Wildcard tous: "*"
    /// - Wildcard sous-domaines: ".exemple.com" match "api.exemple.com", "www.exemple.com", etc.
    pub fn is_host_allowed(&self, host: &str) -> bool {
        // En mode debug, on est plus permissif
        if self.debug {
            return true;
        }

        // Nettoie le host (retire le port si présent)
        let host = host.split(':').next().unwrap_or(host);

        // Vérifie si le host est dans la liste
        self.allowed_hosts.iter().any(|allowed| {
            if allowed == "*" {
                // Wildcard complet (dangereux en production!)
                true
            } else if allowed.starts_with('.') {
                // Wildcard sous-domaines: ".exemple.com"
                // Match: api.exemple.com, www.exemple.com, etc.
                // Match aussi: exemple.com (sans le point)
                // Ne match PAS: malicious-exemple.com (bug de sécurité corrigé)
                if host == &allowed[1..] {
                    true
                } else if host.ends_with(allowed) {
                    // Vérifie que le caractère à l'index où commence la correspondance est un point
                    // pour éviter que "malicious-exemple.com" match ".exemple.com"
                    let match_start = host.len() - allowed.len();
                    // match_start doit être > 0 (pas au début) et le caractère à cet index doit être '.'
                    match_start > 0 && host.as_bytes()[match_start] == b'.'
                } else {
                    false
                }
            } else {
                // Correspondance exacte
                allowed == host
            }
        })
    }

    /// Valide le host depuis les headers HTTP
    pub fn validate(&self, headers: &HeaderMap) -> Result<(), (StatusCode, String)> {
        // Récupère l'en-tête Host
        let host = match headers.get(header::HOST) {
            Some(h) => match h.to_str() {
                Ok(s) => s,
                Err(_) => {
                    return Err((
                        StatusCode::BAD_REQUEST,
                        self.make_error_message("<invalid host header>"),
                    ));
                }
            },
            None => {
                return Err((
                    StatusCode::BAD_REQUEST,
                    self.make_error_message("<no host>"),
                ));
            }
        };

        // Vérifie si le host est autorisé
        if !self.is_host_allowed(host) {
            eprintln!(
                "⚠️  Host non autorisé: '{}'. Hosts autorisés: {:?}",
                host, self.allowed_hosts
            );
            return Err((StatusCode::BAD_REQUEST, self.make_error_message(host)));
        }

        Ok(())
    }

    /// Génère le message d'erreur approprié
    fn make_error_message(&self, host: &str) -> String {
        if self.debug {
            format!(
                "Invalid HTTP_HOST header: '{}'\n\
                You may need to add '{}' to ALLOWED_HOSTS.",
                host, host
            )
        } else {
            "Bad Request".to_string()
        }
    }
}

/// Middleware Axum pour valider les hosts autorisés
///
/// ```rust,no_run
/// # use runique::Settings;
/// # use runique::app::RuniqueApp;
/// # use axum::Router;
/// # async fn doc() -> Result<(), Box<dyn std::error::Error>> {
/// let settings = Settings::default_values();
/// let routes = Router::new();
///
/// // On attend le build, mais pas l'objet final lui-même
/// let app = RuniqueApp::builder(settings).await
///     .routes(routes)
///     .build()
///     .await?;
/// # Ok(())
/// # }
/// ```
pub async fn allowed_hosts_middleware(
    settings: axum::extract::Extension<Arc<Settings>>,
    request: Request,
    next: Next,
) -> Response {
    let validator = AllowedHostsValidator::from_settings(&settings);

    // Valide le host
    if let Err((status, message)) = validator.validate(request.headers()) {
        return (status, message).into_response();
    }

    // Continue vers le prochain middleware
    next.run(request).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exact_match() {
        let validator = AllowedHostsValidator::new(vec!["exemple.com".to_string()], false);

        assert!(validator.is_host_allowed("exemple.com"));
        assert!(!validator.is_host_allowed("www.exemple.com"));
        assert!(!validator.is_host_allowed("malicious.com"));
    }

    #[test]
    fn test_wildcard_subdomain() {
        let validator = AllowedHostsValidator::new(vec![".exemple.com".to_string()], false);

        assert!(validator.is_host_allowed("exemple.com"));
        assert!(validator.is_host_allowed("www.exemple.com"));
        assert!(validator.is_host_allowed("api.exemple.com"));
        assert!(validator.is_host_allowed("admin.api.exemple.com"));
        assert!(!validator.is_host_allowed("malicious.com"));
    }

    #[test]
    fn test_wildcard_all() {
        let validator = AllowedHostsValidator::new(vec!["*".to_string()], false);

        assert!(validator.is_host_allowed("exemple.com"));
        assert!(validator.is_host_allowed("n-importe-quoi.com"));
    }

    #[test]
    fn test_multiple_hosts() {
        let validator = AllowedHostsValidator::new(
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
        let validator = AllowedHostsValidator::new(vec!["exemple.com".to_string()], false);

        assert!(validator.is_host_allowed("exemple.com:8080"));
        assert!(validator.is_host_allowed("exemple.com:443"));
    }

    #[test]
    fn test_debug_mode_allows_all() {
        let validator = AllowedHostsValidator::new(
            vec!["exemple.com".to_string()],
            true, // debug = true
        );

        assert!(validator.is_host_allowed("n-importe-quoi.com"));
        assert!(validator.is_host_allowed("malicious.com"));
    }

    #[test]
    fn test_wildcard_subdomain_security() {
        // Test pour éviter que "malicious-exemple.com" match ".exemple.com"
        let validator = AllowedHostsValidator::new(vec![".exemple.com".to_string()], false);

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
