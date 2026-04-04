//! Implémentation de `RuniqueEngine` — construction, attachement des middlewares, accès aux stores.
use crate::middleware::session::{CleaningMemoryStore, session_db::RuniqueSessionStore};
use crate::utils::aliases::{
    ADb, ARlockmap, ASecurityCsp, ASecurityHosts, ATera, new, new_registry,
};
use axum::{Router, middleware};
use std::sync::{Arc, LazyLock, RwLock};
use tera::Tera;

use crate::config::RuniqueConfig;
// On importe nos nouvelles structures renommées
use crate::middleware::{
    HostPolicy, MiddlewareConfig, SecurityPolicy, allowed_hosts_middleware, csrf_middleware,
    dev_no_cache_middleware, error_handler_middleware, https_redirect_middleware,
    security_headers_middleware,
};

#[cfg(feature = "orm")]
use sea_orm::DatabaseConnection;
/// Machine centrale du framework : regroupe la config, le moteur de templates,
/// la base de données, le registre d'URLs et toutes les politiques de sécurité.
#[derive(Debug)]
pub struct RuniqueEngine {
    /// Configuration générale de l'application.
    pub config: RuniqueConfig,
    /// Instance Tera partagée en lecture/écriture.
    pub tera: ATera,
    #[cfg(feature = "orm")]
    /// Connexion à la base de données (feature `orm`).
    pub db: ADb,
    /// Registre global des routes nommées (reverse URL).
    pub url_registry: ARlockmap,
    /// Interrupteurs middleware (cache, CSP, CSRF, etc.).
    pub features: MiddlewareConfig,
    /// Politique Content Security Policy active.
    pub security_csp: ASecurityCsp,
    /// Politique de validation des hôtes autorisés.
    pub security_hosts: ASecurityHosts,
    /// Store mémoire — sessions anonymes + CSRF.
    pub session_store: LazyLock<RwLock<Option<Arc<CleaningMemoryStore>>>>,
    /// Store DB — sessions authentifiées persistantes (table `eihwaz_sessions`).
    pub session_db_store: LazyLock<RwLock<Option<Arc<RuniqueSessionStore>>>>,
}

impl RuniqueEngine {
    /// Construit un nouveau moteur à partir de la config, de Tera et de la connexion DB.
    #[cfg(feature = "orm")]
    pub fn new(config: RuniqueConfig, tera: Tera, db: DatabaseConnection) -> Self {
        // Chargement unique au démarrage
        let features = MiddlewareConfig::from_env();
        let security_csp = SecurityPolicy::default();
        let security_hosts = HostPolicy::default();

        Self {
            config,
            tera: new(tera),
            db: new(db),
            url_registry: new_registry(),
            features,
            security_csp: new(security_csp),
            security_hosts: new(security_hosts),
            session_store: LazyLock::new(|| RwLock::new(None)),
            session_db_store: LazyLock::new(|| RwLock::new(None)),
        }
    }

    /// Attache les middlewares globaux (HTTPS, hosts, CSRF, cache, CSP, erreurs)
    /// sur le router selon la configuration active.
    pub fn attach_middlewares(engine: Arc<Self>, router: Router) -> Router {
        let mut router = router;
        let f = &engine.features;

        // 0. HTTPS Redirection (Avant tout pour éviter les redirections inutiles)
        if engine.config.security.enforce_https {
            router = router.layer(middleware::from_fn_with_state(
                engine.clone(),
                https_redirect_middleware,
            ));
        }

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

        // 3. Cache (activé via .env)
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
