/// Configuration centralisée de tous les middlewares Runique
///
/// Note: Le CSRF est TOUJOURS activé (imposé par le framework pour la sécurité)
///
/// ## Gestion des pages d'erreur
/// `enable_debug_errors` contrôle si le middleware `error_handler` est ajouté au router.
/// Quand il est actif, les pages d'erreur sont rendues par Tera :
/// - En **développement** (`DEBUG=true` ou `cargo build` sans `--release`) : traces complètes.
/// - En **production** (`cargo build --release`) : pages 404/500 propres sans trace.
///
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MiddlewareConfig {
    pub enable_csp: bool,
    /// Active les headers de sécurité additionnels (HSTS, X-Frame-Options, COEP, COOP, CORP,
    /// Referrer-Policy, Permissions-Policy). Sans effet si `enable_csp` est false.
    ///
    /// Quand `true` : utilise `security_headers_middleware` (CSP + headers additionnels).
    /// Quand `false` : utilise `csp_middleware` (CSP uniquement).
    pub enable_header_security: bool,
    pub enable_host_validation: bool,
    /// Active le middleware `error_handler` qui intercepte les erreurs 4xx/5xx.
    /// Désactiver uniquement si vous gérez les erreurs manuellement dans chaque handler.
    pub enable_debug_errors: bool,
    pub enable_cache: bool,
}

impl Default for MiddlewareConfig {
    fn default() -> Self {
        Self {
            enable_csp: true,
            enable_header_security: false,
            enable_host_validation: true,
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
            // CSP configuré uniquement via le builder, pas le .env
            enable_csp: false,
            enable_header_security: false,
            enable_host_validation: get_bool("RUNIQUE_ENABLE_HOST_VALIDATION", true),
            enable_debug_errors: get_bool("RUNIQUE_ENABLE_DEBUG_ERRORS", true),
            enable_cache: get_bool("RUNIQUE_ENABLE_CACHE", true),
        }
    }

    /// Configuration pour production (sécurité maximale)
    pub fn production() -> Self {
        Self {
            enable_csp: true,
            enable_header_security: false,
            enable_host_validation: true,
            enable_debug_errors: true,
            enable_cache: true,
        }
    }

    /// Configuration pour développement (plus permissif)
    pub fn development() -> Self {
        Self {
            enable_csp: false, // CSP désactivé pour faciliter le dev
            enable_header_security: false,
            enable_host_validation: false, // Désactivé en dev (AllowedHostsValidator.debug=true anyway)
            enable_debug_errors: true,     // Pages de debug activées
            enable_cache: false,           // Pas de cache en dev
        }
    }

    /// Configuration pour API (minimal)
    pub fn api() -> Self {
        Self {
            enable_csp: false, // Pas de CSP pour API
            enable_header_security: false,
            enable_host_validation: true, // Important pour API (protection SSRF)
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
