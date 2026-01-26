use axum::{middleware, Router};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use tera::Tera;

use crate::config::RuniqueConfig;
// On importe nos nouvelles structures renommées
use crate::middleware::{
    allowed_hosts_middleware, csrf_middleware, dev_no_cache_middleware, error_handler_middleware,
    security_headers_middleware, HostPolicy, MiddlewareConfig, SecurityPolicy,
};

#[cfg(feature = "orm")]
use sea_orm::DatabaseConnection;

/// Machine centrale de l'application
pub struct RuniqueEngine {
    pub config: RuniqueConfig,
    pub tera: Arc<Tera>,
    #[cfg(feature = "orm")]
    pub db: Arc<DatabaseConnection>,
    pub url_registry: Arc<RwLock<HashMap<String, String>>>,

    // Les interrupteurs (La Porte)
    pub features: MiddlewareConfig,

    // Les politiques (Les Meubles)
    pub security_csp: Arc<SecurityPolicy>,
    pub security_hosts: Arc<HostPolicy>,
}

impl RuniqueEngine {
    #[cfg(feature = "orm")]
    pub fn new(config: RuniqueConfig, tera: Tera, db: DatabaseConnection) -> Self {
        // Chargement unique au démarrage
        let features = MiddlewareConfig::from_env();
        let security_csp = SecurityPolicy::from_env();
        let security_hosts = HostPolicy::from_env();

        Self {
            config,
            tera: Arc::new(tera),
            db: Arc::new(db),
            url_registry: Arc::new(RwLock::new(HashMap::new())),
            features,
            security_csp: Arc::new(security_csp),
            security_hosts: Arc::new(security_hosts),
        }
    }

    pub fn attach_middlewares(engine: Arc<Self>, router: Router) -> Router {
        let mut router = router;
        let f = &engine.features;

        // 1. Validation des Hosts (La toute première ligne de défense)
        if f.enable_host_validation {
            router = router.layer(middleware::from_fn_with_state(
                engine.clone(),
                allowed_hosts_middleware,
            ));
        }

        // 2. CSRF (Sécurité par design : intégré via ExtractForm + Signal de validation)
        // Note : On garde le middleware si tu as une logique globale,
        // sinon l'ExtractForm s'en occupe comme on l'a prévu.
        router = router.layer(middleware::from_fn_with_state(
            engine.clone(),
            csrf_middleware,
        ));

        // 3. Cache (piloté par le .env)
        if !f.enable_cache {
            router = router.layer(middleware::from_fn_with_state(
                engine.clone(),
                dev_no_cache_middleware,
            ));
        }

        // 4. Security Headers (CSP, HSTS, etc.)
        if f.enable_csp {
            router = router.layer(middleware::from_fn_with_state(
                engine.clone(),
                security_headers_middleware,
            ));
        }

        // 5. Error Handler (En dernier pour attraper les erreurs des autres)
        if f.enable_debug_errors {
            router = router.layer(middleware::from_fn(error_handler_middleware));
        }

        router
    }
}
