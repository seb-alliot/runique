use crate::utils::aliases::{
    new, new_registry, ADb, ARlockmap, ASecurityCsp, ASecurityHosts, ATera,
};
use axum::{middleware, Router};
use std::sync::Arc;
use tera::Tera;

use crate::config::RuniqueConfig;
// On importe nos nouvelles structures renommées
use crate::middleware::{
    allowed_hosts_middleware, csrf_middleware, dev_no_cache_middleware, error_handler_middleware,
    security_headers_middleware, HostPolicy, MiddlewareConfig, SecurityPolicy,
};

#[cfg(feature = "orm")]
use sea_orm::DatabaseConnection;
#[derive(Debug)]
/// Machine centrale de l'application
pub struct RuniqueEngine {
    pub config: RuniqueConfig,
    pub tera: ATera,
    #[cfg(feature = "orm")]
    pub db: ADb,
    pub url_registry: ARlockmap,

    // Les interrupteurs (La Porte)
    pub features: MiddlewareConfig,

    // Les politiques (Les Meubles)
    pub security_csp: ASecurityCsp,
    pub security_hosts: ASecurityHosts,
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
            tera: new(tera),
            db: new(db),
            url_registry: new_registry(),
            features,
            security_csp: new(security_csp),
            security_hosts: new(security_hosts),
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
