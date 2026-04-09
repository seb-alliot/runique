//! Builder intelligent de l'application Runique : collecte, valide et assemble
//! tous les composants (core, middlewares, routes, admin, fichiers statiques).
use axum::Router;
use std::sync::Arc;
use tower_sessions::cookie::time::Duration;

use super::error_build::BuildError;
use super::runique_app::RuniqueApp;
use super::staging::{AdminStaging, CoreStaging, MiddlewareStaging, StaticStaging};
use super::templates::TemplateLoader;
use crate::admin::build_admin_router;
use crate::config::RuniqueConfig;
use crate::engine::RuniqueEngine;
use crate::macros::add_urls;
use crate::middleware::HostPolicy;
use crate::middleware::auth::{
    PasswordResetAdapter, PasswordResetConfig, PasswordResetStaging, UserEntity,
};
#[cfg(feature = "orm")]
use crate::middleware::session::session_db::RuniqueSessionStore;
use crate::utils::aliases::new;
use crate::utils::runique_log::{RuniqueLog, log_init};
use axum::http::{HeaderName, HeaderValue};
use tower_http::{services::ServeDir, set_header::SetResponseHeaderLayer};

#[cfg(feature = "orm")]
use crate::db::DatabaseConfig;
#[cfg(feature = "orm")]
use sea_orm::DatabaseConnection;

// ═══════════════════════════════════════════════════════════════
// Builder Intelligent — Innovation Runique
// ═══════════════════════════════════════════════════════════════
//
// Premier framework web à combiner flexibilité d'écriture
// et rigueur d'exécution via un pipeline de validation
// + réorganisation automatique des middlewares par slots.
//
//   Flexibilité (Staging) + Validation (Pipeline)
//   + Réorganisation (Slots) = Builder Intelligent
//
// Le développeur configure dans l'ordre qui lui semble logique.
// Chaque staging valide ses composants, puis réorganise
// automatiquement pour garantir un démarrage optimal.
//
// ═══════════════════════════════════════════════════════════════
//
// USAGE :
//
//   RuniqueApp::builder(config)
//       .core(|c| c.with_database(db))
//       .routes(router)
//       .static_files(|s| s.disable())
//       .middleware(|m| {
//           m.with_csp(|c| {
//               c.with_header_security(true)
//                .with_nonce(true)
//                .scripts(vec!["'self'"])
//           })
//           .add_custom(my_auth_middleware)
//       })
//       .build().await?
//
//   RuniqueApp::builder(config)
//       .with_database(db)
//       .routes(router)
//       .statics()
//       .middleware(|m| m.with_csp(|c| c.with_header_security(true)))
//       .build().await?
//
// ═══════════════════════════════════════════════════════════════

/// Intelligent application builder for Runique
///
#[doc = include_str!("../../doc-tests/builder/builder_basic.md")]
pub struct RuniqueAppBuilder {
    config: RuniqueConfig,
    core: CoreStaging,
    middleware: MiddlewareStaging,
    statics: StaticStaging,
    router: Option<Router>,
    admin: AdminStaging,
    password_reset: Option<PasswordResetStaging>,
}

impl RuniqueAppBuilder {
    /// Crée un nouveau builder intelligent avec la configuration donnée
    ///
    /// Le `MiddlewareConfig` est récupéré directement depuis `RuniqueConfig`
    /// (loaded via `.env` or `from_env()`). The staging uses it as a base
    /// et le dev peut le surcharger ensuite via `.middleware(|m| ...)`.
    pub fn new(config: RuniqueConfig) -> Self {
        let middleware = MiddlewareStaging::from_config(&config);
        Self {
            config,
            core: CoreStaging::new(),
            middleware,
            statics: StaticStaging::new(),
            router: None,
            admin: AdminStaging::new(),
            password_reset: None,
        }
    }

    // PHASE 1 : COLLECTE FLEXIBLE
    //
    // Chaque méthode stocke la donnée sans l'exécuter.
    // Peu importe l'ordre d'appel par un dév.

    // CORE — Base de données et composants fondamentaux

    /// Configure le core via une closure.
    ///
    /// Même principe que `.middleware()` : le dev configure
    /// dans l'ordre qu'il veut, le staging valide au build.
    ///
    /// # Exemple
    /// ```rust,ignore
    /// .core(|c| c.with_database(db))
    /// .core(|c| c.with_database_config(db_config))
    /// ```
    pub fn core(mut self, f: impl FnOnce(CoreStaging) -> CoreStaging) -> Self {
        self.core = f(self.core);
        self
    }

    /// Raccourci : ajoute une connexion DB déjà établie sans passer par `.core()`
    ///
    /// ```rust,ignore
    /// let db = DatabaseConfig::from_env()?.build().connect().await?;
    /// RuniqueApp::builder(config).with_database(db)
    /// ```
    #[cfg(feature = "orm")]
    pub fn with_database(mut self, db: DatabaseConnection) -> Self {
        self.core = self.core.with_database(db);
        self
    }

    /// Raccourci : ajoute une configuration DB — connexion auto pendant `build()`
    ///
    /// ```rust,ignore
    /// let db_config = DatabaseConfig::from_env()?.build();
    /// RuniqueApp::builder(config).with_database_config(db_config)
    /// ```
    #[cfg(feature = "orm")]
    pub fn with_database_config(mut self, config: DatabaseConfig) -> Self {
        self.core = self.core.with_database_config(config);
        self
    }

    /// Configure les logs Runique par catégorie.
    ///
    /// Chaque catégorie est désactivée par défaut. Appeler la méthode
    /// correspondante avec un niveau tracing active la catégorie.
    ///
    /// # Exemple
    /// ```rust,ignore
    /// use tracing::Level;
    ///
    /// RuniqueApp::builder(config)
    ///     .with_log(|l| l
    ///         .csrf(Level::WARN)
    ///         .exclusive_login(Level::INFO)
    ///     )
    /// ```
    pub fn with_log(mut self, f: impl FnOnce(RuniqueLog) -> RuniqueLog) -> Self {
        self.config.log = f(RuniqueLog::new());
        self
    }

    // ROUTES

    /// Définit les routes de l'application
    pub fn routes(mut self, router: Router) -> Self {
        self.router = Some(router);
        self
    }

    // MIDDLEWARE — Réorganisation automatique par slots

    /// Configure les middlewares via une closure.
    ///
    /// L'ordre des appels à l'intérieur de la closure n'a aucune importance :
    /// le framework appliquera les middlewares dans l'ordre optimal garanti
    /// grâce au système de slots.
    ///
    /// CSRF dépend de Session ? Le staging le sait et réordonne automatiquement.
    ///
    /// # Exemple
    /// ```rust,ignore
    /// .middleware(|m| {
    ///     m.with_csp(true)
    ///      .with_session_store(RedisStore::new(client))
    ///      .with_session_duration(Duration::hours(2))
    ///      .add_custom(my_auth_layer)
    /// })
    /// ```
    pub fn middleware(mut self, f: impl FnOnce(MiddlewareStaging) -> MiddlewareStaging) -> Self {
        self.middleware = f(self.middleware);
        self
    }

    /// Raccourci : configure la durée de session sans passer par `.middleware()`
    pub fn with_session_duration(mut self, duration: Duration) -> Self {
        self.middleware = self.middleware.with_session_duration(duration);
        self
    }

    /// Raccourci : active/désactive les pages d'erreur de debug
    pub fn with_error_handler(mut self, enable: bool) -> Self {
        self.middleware = self.middleware.with_debug_errors(enable);
        self
    }

    // FICHIERS STATIQUES

    /// Configure les fichiers statiques via une closure.
    ///
    /// Même principe que `.middleware()` et `.core()` :
    /// configuration flexible, validation au build.
    ///
    /// # Exemple
    /// ```rust,ignore
    /// .static_files(|s| s.disable())
    /// ```
    pub fn static_files(mut self, f: impl FnOnce(StaticStaging) -> StaticStaging) -> Self {
        self.statics = f(self.statics);
        self
    }

    /// Configure le mailer SMTP manuellement
    ///
    /// ```rust,ignore
    /// builder::new(config)
    ///     .with_mailer(MailerConfig { host: "smtp.example.com".into(), port: 587, ... })
    /// ```
    pub fn with_mailer(self, config: crate::utils::mailer::MailerConfig) -> Self {
        crate::utils::mailer::mailer_init(config);
        self
    }

    /// Configure le mailer depuis les variables d'environnement
    /// (SMTP_HOST, SMTP_USER, SMTP_PASS, SMTP_FROM, SMTP_PORT, SMTP_STARTTLS)
    pub fn with_mailer_from_env(self) -> Self {
        crate::utils::mailer::mailer_init_from_env();
        self
    }

    /// Raccourci : active le service de fichiers statiques (activé par défaut)
    pub fn statics(mut self) -> Self {
        self.statics = self.statics.enable();
        self
    }

    /// Raccourci : désactive le service de fichiers statiques
    pub fn no_statics(mut self) -> Self {
        self.statics = self.statics.disable();
        self
    }

    // ═══════════════════════════════════════════════════════════
    // ADMIN PANEL
    // ═══════════════════════════════════════════════════════════

    /// Configure et active l'AdminPanel via une closure.
    ///
    /// ```rust,ignore
    /// .with_admin(|a| a
    ///     .prefix("/admin")
    ///     .hot_reload(is_debug())
    ///     .site_title("Mon Admin")
    /// )
    /// ```
    pub fn with_admin(mut self, f: impl FnOnce(AdminStaging) -> AdminStaging) -> Self {
        self.admin = f(self.admin.enable());
        self
    }

    // ═══════════════════════════════════════════════════════════
    // RESET PASSWORD
    // ═══════════════════════════════════════════════════════════

    /// Active le flow reset password built-in pour une entité donnée.
    ///
    /// Enregistre automatiquement deux routes :
    ///   - `{config.forgot_route}` — formulaire email (étape 1)
    ///   - `{config.reset_route}/{token}/{encrypted_email}` — nouveau mdp (étape 2)
    ///
    /// Exemple minimal (entité built-in) :
    /// ```rust,ignore
    /// .with_password_reset::<BuiltinUserEntity>(|pr| pr)
    /// ```
    ///
    /// Avec config personnalisée :
    /// ```rust,ignore
    /// .with_password_reset::<MyEntity>(|pr| pr
    ///     .forgot_route("/mot-de-passe-oublie")
    ///     .reset_route("/reinitialiser")
    ///     .base_url("https://monsite.fr")
    /// )
    /// ```
    pub fn with_password_reset<E: UserEntity + 'static>(
        mut self,
        f: impl FnOnce(PasswordResetConfig) -> PasswordResetConfig,
    ) -> Self {
        let config = f(PasswordResetConfig::default());
        self.password_reset = Some(PasswordResetStaging {
            handler: Box::new(PasswordResetAdapter::<E>::new()),
            config,
        });
        self
    }

    // ═══════════════════════════════════════════════════════════
    // PHASE 2 : VALIDATION + CONSTRUCTION (pipeline strict)
    //
    // Comme Prisme (formulaires) :
    // 1. validate() — vérifie chaque staging + dépendances croisées
    // 2. all_ready() — signal OK
    // 3. Construction dans l'ordre STRICT garanti
    // 4. MiddlewareStaging réorganise par slots et applique
    // ═══════════════════════════════════════════════════════════

    /// Valide et construit l'application.
    ///
    /// # Pipeline de construction
    /// 1. **Validation** de tous les composants (Core, Middleware, Statics)
    /// 2. **Construction** du Core (Templates → Engine → URLs)
    /// 3. **Réorganisation** automatique des middlewares par slots
    /// 4. **Application** des fichiers statiques (si activés)
    pub async fn build(mut self) -> Result<RuniqueApp, BuildError> {
        // ═══════════════════════════════════════
        // ÉTAPE 0 : TRACING (avant tout le reste)
        // ═══════════════════════════════════════
        self.config.log.init_subscriber();

        // ═══════════════════════════════════════
        // ÉTAPE 1 : VALIDATION (comme Prisme)
        // ═══════════════════════════════════════
        self.validate()?;

        if !self.all_ready() {
            return Err(BuildError::validation(
                "Un ou plusieurs composants ne sont pas prêts pour la construction",
            ));
        }

        // ═══════════════════════════════════════
        // ÉTAPE 2 : CONNEXION DB (si DatabaseConfig fourni)
        //
        // Deux chemins possibles :
        //   1. with_database(db)        → déjà connecté, on prend tel quel
        //   2. with_database_config(cfg) → connect() pendant le build
        // ═══════════════════════════════════════
        #[cfg(feature = "orm")]
        let db = self.core.connect().await?;

        // ═══════════════════════════════════════
        // ÉTAPE 3 : DÉSTRUCTURATION
        // ═══════════════════════════════════════
        let config = self.config;
        let url_registry = self.core.url_registry;
        let mut middleware = self.middleware;
        let statics_enabled = self.statics.enabled;
        let router = self.router;

        // ═══════════════════════════════════════
        // ÉTAPE 4 : CONSTRUCTION CORE
        // Ordre strict : Templates → Config → Engine → URLs
        // ═══════════════════════════════════════

        // A. Templates (Tera) — toujours en premier
        let tera = new(TemplateLoader::init(&config, url_registry.clone())
            .map_err(|e| BuildError::template(e.to_string()))?);

        let config = new(config);
        log_init(config.log.clone());
        crate::utils::password::password_init(config.password.clone());

        // B. Engine (cœur de l'application)
        let engine = new(RuniqueEngine {
            config: (*config).clone(),
            tera: tera.clone(),
            #[cfg(feature = "orm")]
            db: new(db),
            features: {
                let mut f = middleware.features.clone();
                f.exclusive_login = middleware.exclusive_login;
                f
            },
            url_registry,
            security_csp: {
                let mut policy = middleware.security_policy.take().unwrap_or_default();
                if self.admin.enabled {
                    policy.merge_htmx_hashes();
                }
                new(policy)
            },
            security_hosts: new(HostPolicy::new(
                middleware.allowed_hosts.clone(),
                middleware.features.enable_host_validation,
            )),
            session_store: std::sync::LazyLock::new(|| std::sync::RwLock::new(None)),
            session_db_store: std::sync::LazyLock::new(|| std::sync::RwLock::new(None)),
        });

        // C. Enregistrement des URLs (urlpatterns!)
        add_urls(&engine);

        // ═══════════════════════════════════════
        // ═══════════════════════════════════════
        // ÉTAPE 4b : ADMIN PANEL — mergé AVANT la stack middleware
        //
        // .layer() en Axum ne couvre que les routes déjà présentes
        // sur le router au moment de l'appel.
        // Merger après = routes admin sans Session/CSRF/Extensions.
        // ═══════════════════════════════════════

        let router = router.unwrap_or_default();

        // ═══════════════════════════════════════
        // ÉTAPE 4b.1 : RESET PASSWORD (avant middleware, comme admin)
        // ═══════════════════════════════════════
        let router = if let Some(pr) = self.password_reset {
            let pr_router = pr.handler.build_router(std::sync::Arc::new(pr.config));
            router.merge(pr_router)
        } else {
            router
        };

        let router = if self.admin.enabled {
            let admin_prefix = self.admin.config.prefix.trim_end_matches('/').to_string();
            let robots_txt = self.admin.robots_txt;
            let admin_router = build_admin_router(self.admin, engine.db.clone());
            add_urls(&engine);
            let mut r = router.merge(admin_router);
            if robots_txt {
                r = r.route(
                    "/robots.txt",
                    axum::routing::get(move || {
                        let body = format!("User-agent: *\nDisallow: {}/\n", admin_prefix);
                        async move { body }
                    }),
                );
            }
            r
        } else {
            router
        };

        // ═══════════════════════════════════════
        // ÉTAPE 5 : MIDDLEWARE STAGING
        //
        // Appliqué sur toutes les routes (dev + admin).
        // Le staging réorganise automatiquement par slots :
        //   Extensions → Session → CSRF → CSP → Host
        // ═══════════════════════════════════════

        let _exclusive_login = middleware.exclusive_login;
        let (router, session_store) =
            middleware.apply_to_router(router, config, engine.clone(), tera);
        if let Some(store) = session_store {
            if let Ok(mut guard) = engine.session_store.write() {
                *guard = Some(store);
            }
        }

        // Store DB sessions — initialisé si une DB est disponible
        #[cfg(feature = "orm")]
        {
            let db_store = RuniqueSessionStore::new(engine.db.clone());
            if let Ok(mut guard) = engine.session_db_store.write() {
                *guard = Some(Arc::new(db_store));
            }
        }

        // ═══════════════════════════════════════
        // ÉTAPE 6 : FICHIERS STATIQUES (conditionnel)
        // ═══════════════════════════════════════

        let router = if statics_enabled {
            Self::attach_static_files(router, &engine.config)
        } else {
            router
        };

        Ok(RuniqueApp { engine, router })
    }

    // ═══════════════════════════════════════════════════════════
    // VALIDATION INTERNE
    // ═══════════════════════════════════════════════════════════

    /// Validation individuelle de chaque staging, puis croisée
    fn validate(&self) -> Result<(), BuildError> {
        // Validation individuelle (comme field.validate() dans Prisme)
        self.core.validate()?;
        self.middleware.validate()?;
        self.statics.validate()?;
        self.admin.validate()?;

        // Validation croisée (dépendances entre composants)
        self.cross_validate()?;

        Ok(())
    }

    /// Vérifie les dépendances entre composants
    fn cross_validate(&self) -> Result<(), BuildError> {
        // Futures validations inter-composants :
        //
        // - host_validation activé → ALLOWED_HOSTS défini ?
        // - enable_debug_errors en production → warning
        // - CSP strict + session Memory → warning
        Ok(())
    }

    /// Vérifie que tous les composants sont prêts
    fn all_ready(&self) -> bool {
        self.core.is_ready()
            && self.middleware.is_ready()
            && self.statics.is_ready()
            && self.admin.is_ready()
    }

    // ═══════════════════════════════════════════════════════════
    // FICHIERS STATIQUES
    // ═══════════════════════════════════════════════════════════

    /// Attache les routes de fichiers statiques au route
    fn attach_static_files(mut router: Router, config: &RuniqueConfig) -> Router {
        let static_headers = tower::ServiceBuilder::new()
            .layer(SetResponseHeaderLayer::if_not_present(
                HeaderName::from_static("x-content-type-options"),
                HeaderValue::from_static("nosniff"),
            ))
            .layer(SetResponseHeaderLayer::if_not_present(
                HeaderName::from_static("strict-transport-security"),
                HeaderValue::from_static("max-age=31536000; includeSubDomains; preload"),
            ))
            .layer(SetResponseHeaderLayer::if_not_present(
                HeaderName::from_static("x-frame-options"),
                HeaderValue::from_static("DENY"),
            ))
            .layer(SetResponseHeaderLayer::if_not_present(
                HeaderName::from_static("referrer-policy"),
                HeaderValue::from_static("strict-origin-when-cross-origin"),
            ));

        router = router
            .nest_service(
                &config.static_files.static_url,
                static_headers
                    .clone()
                    .service(ServeDir::new(&config.static_files.staticfiles_dirs)),
            )
            .nest_service(
                &config.static_files.media_url,
                static_headers
                    .clone()
                    .service(ServeDir::new(&config.static_files.media_root)),
            );

        if !config.static_files.static_runique_url.is_empty() {
            router = router.nest_service(
                &config.static_files.static_runique_url,
                static_headers.service(ServeDir::new(&config.static_files.static_runique_path)),
            );
        }

        router
    }
}
