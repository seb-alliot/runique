/// Configuration centralisée de tous les middlewares Runique
///
/// Note: Le CSRF est TOUJOURS activé (imposé par le framework pour la sécurité)
///
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MiddlewareConfig {
    pub enable_csp: bool,
    pub enable_host_validation: bool,
    pub sanitize_inputs: bool,
    pub enable_debug_errors: bool,
    pub enable_cache: bool,
}

impl Default for MiddlewareConfig {
    fn default() -> Self {
        Self {
            // Par défaut: tous les middlewares de sécurité activés
            enable_csp: true,
            enable_host_validation: true,
            sanitize_inputs: true,
            enable_debug_errors: true,
            enable_cache: true,
        }
    }
}

impl MiddlewareConfig {
    pub fn from_env() -> Self {
        let get_bool = |key: &str, default: bool| {
            std::env::var(key)
                .map(|v| v.parse::<bool>().unwrap_or(default))
                .unwrap_or(default)
        };

        Self {
            enable_csp: get_bool("RUNIQUE_ENABLE_CSP", true),
            enable_host_validation: get_bool("RUNIQUE_ENABLE_HOST_VALIDATION", true),
            sanitize_inputs: get_bool("RUNIQUE_ENABLE_SANITIZER", true),
            enable_debug_errors: get_bool("RUNIQUE_ENABLE_DEBUG_ERRORS", true),
            enable_cache: get_bool("RUNIQUE_ENABLE_CACHE", true),
        }
    }

    /// Configuration pour production (sécurité maximale)
    pub fn production() -> Self {
        Self {
            enable_csp: true,
            enable_host_validation: true,
            sanitize_inputs: true,
            enable_debug_errors: true,
            enable_cache: true,
        }
    }

    /// Configuration pour développement (plus permissif)
    pub fn development() -> Self {
        Self {
            enable_csp: false,             // CSP désactivé pour faciliter le dev
            enable_host_validation: false, // Désactivé en dev (AllowedHostsValidator.debug=true anyway)
            sanitize_inputs: true,         // Garder quand même
            enable_debug_errors: true,     // Pages de debug activées
            enable_cache: false,           // Pas de cache en dev
        }
    }

    /// Configuration pour API (minimal)
    pub fn api() -> Self {
        Self {
            enable_csp: false,            // Pas de CSP pour API
            enable_host_validation: true, // Important pour API (protection SSRF)
            sanitize_inputs: true,        // Important pour API
            enable_debug_errors: true,
            enable_cache: true,
        }
    }

    /// Builder pattern pour customisation
    pub fn custom() -> Self {
        Self::default()
    }

    // Méthodes chainables pour configuration fine

    pub fn with_csp(mut self, enable: bool) -> Self {
        self.enable_csp = enable;
        self
    }

    pub fn with_sanitizer(mut self, enable: bool) -> Self {
        self.sanitize_inputs = enable;
        self
    }

    pub fn with_debug_errors(mut self, enable: bool) -> Self {
        self.enable_debug_errors = enable;
        self
    }

    pub fn with_cache(mut self, enable: bool) -> Self {
        self.enable_cache = enable;
        self
    }

    pub fn with_host_validation(mut self, enable: bool) -> Self {
        self.enable_host_validation = enable;
        self
    }
}
