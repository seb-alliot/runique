/// Configuration centralisée de tous les middlewares Runique
///
/// Note: Le CSRF est TOUJOURS activé (imposé par le framework pour la sécurité)
#[derive(Debug, Clone)]
pub struct MiddlewareConfig {
    // Sécurité
    pub enable_csp: bool,           // CSP + security headers (avec nonce)
    pub enable_allowed_hosts: bool, // Validation des hosts autorisés

    // Input/Output
    pub sanitize_inputs: bool,      // Sanitize automatique des inputs
    pub enable_error_handler: bool, // Pages d'erreur avec debug

    // Performance
    pub enable_cache: bool, // No-cache en dev (via dev_no_cache_middleware)
}

impl Default for MiddlewareConfig {
    fn default() -> Self {
        Self {
            // Par défaut: tous les middlewares de sécurité activés
            enable_csp: true,
            enable_allowed_hosts: true,
            sanitize_inputs: true,
            enable_error_handler: true,
            enable_cache: true,
        }
    }
}

impl MiddlewareConfig {
    /// Configuration pour production (sécurité maximale)
    pub fn production() -> Self {
        Self {
            enable_csp: true,
            enable_allowed_hosts: true,
            sanitize_inputs: true,
            enable_error_handler: true,
            enable_cache: true,
        }
    }

    /// Configuration pour développement (plus permissif)
    pub fn development() -> Self {
        Self {
            enable_csp: false,           // CSP désactivé pour faciliter le dev
            enable_allowed_hosts: false, // Désactivé en dev (AllowedHostsValidator.debug=true anyway)
            sanitize_inputs: true,       // Garder quand même
            enable_error_handler: true,  // Pages de debug activées
            enable_cache: false,         // Pas de cache en dev
        }
    }

    /// Configuration pour API (minimal)
    pub fn api() -> Self {
        Self {
            enable_csp: false,          // Pas de CSP pour API
            enable_allowed_hosts: true, // Important pour API (protection SSRF)
            sanitize_inputs: true,      // Important pour API
            enable_error_handler: true,
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

    pub fn with_error_handler(mut self, enable: bool) -> Self {
        self.enable_error_handler = enable;
        self
    }

    pub fn with_cache(mut self, enable: bool) -> Self {
        self.enable_cache = enable;
        self
    }

    pub fn with_allowed_hosts(mut self, enable: bool) -> Self {
        self.enable_allowed_hosts = enable;
        self
    }
}
