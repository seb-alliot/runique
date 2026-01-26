use tower_sessions::cookie::time::Duration;

/// Configuration centralisée de tous les middlewares Runique
///
/// Note: Le CSRF est TOUJOURS activé (imposé par le framework pour la sécurité)
#[derive(Debug, Clone)]
pub struct MiddlewareConfig {
    // Sécurité
    pub enable_csp: bool,           // CSP + security headers (avec nonce)
    pub enable_allowed_hosts: bool, // Validation des hosts autorisés

    // Input/Output
    pub enable_sanitizer: bool,     // Sanitize automatique des inputs
    pub enable_error_handler: bool, // Pages d'erreur avec debug

    // Performance
    pub enable_cache: bool, // No-cache en dev (via dev_no_cache_middleware)

    // Session
    pub session_duration: Duration,
}

impl Default for MiddlewareConfig {
    fn default() -> Self {
        Self {
            // Par défaut: tous les middlewares de sécurité activés
            enable_csp: true,
            enable_allowed_hosts: true,
            enable_sanitizer: true,
            enable_error_handler: true,
            enable_cache: true,
            session_duration: Duration::seconds(86400), // 24h
        }
    }
}

impl MiddlewareConfig {
    /// Configuration pour production (sécurité maximale)
    pub fn production() -> Self {
        Self {
            enable_csp: true,
            enable_allowed_hosts: true,
            enable_sanitizer: true,
            enable_error_handler: true,
            enable_cache: true,
            session_duration: Duration::seconds(3600), // 1h en prod
        }
    }

    /// Configuration pour développement (plus permissif)
    pub fn development() -> Self {
        Self {
            enable_csp: false,                          // CSP désactivé pour faciliter le dev
            enable_allowed_hosts: false, // Désactivé en dev (AllowedHostsValidator.debug=true anyway)
            enable_sanitizer: true,      // Garder quand même
            enable_error_handler: true,  // Pages de debug activées
            enable_cache: false,         // Pas de cache en dev
            session_duration: Duration::seconds(86400), // 24h en dev
        }
    }

    /// Configuration pour API (minimal)
    pub fn api() -> Self {
        Self {
            enable_csp: false,          // Pas de CSP pour API
            enable_allowed_hosts: true, // Important pour API (protection SSRF)
            enable_sanitizer: true,     // Important pour API
            enable_error_handler: true,
            enable_cache: true,
            session_duration: Duration::seconds(3600),
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
        self.enable_sanitizer = enable;
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

    pub fn with_session_duration(mut self, duration: Duration) -> Self {
        self.session_duration = duration;
        self
    }

    pub fn with_allowed_hosts(mut self, enable: bool) -> Self {
        self.enable_allowed_hosts = enable;
        self
    }
}
