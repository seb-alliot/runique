//! Intelligent Runique application builder: collects, validates, and assembles
//! all components (core, middlewares, routes, admin, static files).
use axum::Router;
use std::sync::Arc;
use tower_sessions::cookie::time::Duration;

use super::error_build::BuildError;
use super::runique_app::RuniqueApp;
use super::staging::{AdminStaging, CoreStaging, MiddlewareStaging, StaticStaging};
use super::templates::TemplateLoader;
use crate::admin::build_admin_router;
use crate::auth::{
    PasswordResetAdapter, PasswordResetConfig, PasswordResetStaging, session::UserEntity,
};
use crate::config::RuniqueConfig;
use crate::engine::RuniqueEngine;
use crate::macros::add_urls;
use crate::middleware::HostPolicy;
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
// Intelligent Builder — Runique Innovation
// ═══════════════════════════════════════════════════════════════
//
// First web framework to combine writing flexibility
// and execution rigor via a validation pipeline
// + automatic middleware reorganization by slots.
//
//   Flexibility (Staging) + Validation (Pipeline)
//   + Reorganization (Slots) = Intelligent Builder
//
// The developer configures in the order that seems logical to them.
// Each staging validates its components, then automatically reorganizes
// to guarantee an optimal startup.
//
// ═══════════════════════════════════════════════════════════════
//
// USAGE:
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
    /// Creates a new intelligent builder with the given configuration
    ///
    /// `MiddlewareConfig` is retrieved directly from `RuniqueConfig`
    /// (loaded via `.env` or `from_env()`). The staging uses it as a base
    /// and the dev can then override it via `.middleware(|m| ...)`.
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

    // PHASE 1: FLEXIBLE COLLECTION
    //
    // Each method stores the data without executing it.
    // Regardless of the call order by a dev.

    // CORE — Database and fundamental components

    /// Configures the core via a closure.
    ///
    /// Same principle as `.middleware()`: the dev configures
    /// in the order they want, the staging validates at build.
    ///
    /// # Example
    /// ```rust,ignore
    /// .core(|c| c.with_database(db))
    /// .core(|c| c.with_database_config(db_config))
    /// ```
    pub fn core(mut self, f: impl FnOnce(CoreStaging) -> CoreStaging) -> Self {
        self.core = f(self.core);
        self
    }

    /// Shortcut: adds an already established DB connection without going through `.core()`
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

    /// Shortcut: adds a DB configuration — auto-connection during `build()`
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

    /// Configures Runique logs by category.
    ///
    /// Each category is disabled by default. Calling the corresponding
    /// method with a tracing level enables the category.
    ///
    /// # Example
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

    /// Defines the application routes
    pub fn routes(mut self, router: Router) -> Self {
        self.router = Some(router);
        self
    }

    // MIDDLEWARE — Automatic reorganization by slots

    /// Configures middlewares via a closure.
    ///
    /// The order of calls inside the closure does not matter:
    /// the framework will apply middlewares in the optimal guaranteed order
    /// thanks to the slots system.
    ///
    /// CSRF depends on Session? The staging knows it and reorders automatically.
    ///
    /// # Example
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

    /// Shortcut: configures the session duration without going through `.middleware()`
    pub fn with_session_duration(mut self, duration: Duration) -> Self {
        self.middleware = self.middleware.with_session_duration(duration);
        self
    }

    /// Shortcut: enables/disables debug error pages
    pub fn with_error_handler(mut self, enable: bool) -> Self {
        self.middleware = self.middleware.with_debug_errors(enable);
        self
    }

    // STATIC FILES

    /// Configures static files via a closure.
    ///
    /// Same principle as `.middleware()` and `.core()`:
    /// flexible configuration, validation at build.
    ///
    /// # Example
    /// ```rust,ignore
    /// .static_files(|s| s.disable())
    /// ```
    pub fn static_files(mut self, f: impl FnOnce(StaticStaging) -> StaticStaging) -> Self {
        self.statics = f(self.statics);
        self
    }

    /// Configures the SMTP mailer manually
    ///
    /// ```rust,ignore
    /// builder::new(config)
    ///     .with_mailer(MailerConfig { host: "smtp.example.com".into(), port: 587, ... })
    /// ```
    pub fn with_mailer(self, config: crate::utils::mailer::MailerConfig) -> Self {
        crate::utils::mailer::mailer_init(config);
        self
    }

    /// Configures the mailer from environment variables
    /// (SMTP_HOST, SMTP_USER, SMTP_PASS, SMTP_FROM, SMTP_PORT, SMTP_STARTTLS)
    pub fn with_mailer_from_env(self) -> Self {
        crate::utils::mailer::mailer_init_from_env();
        self
    }

    /// Shortcut: enables the static files service (enabled by default)
    pub fn statics(mut self) -> Self {
        self.statics = self.statics.enable();
        self
    }

    /// Shortcut: disables the static files service
    pub fn no_statics(mut self) -> Self {
        self.statics = self.statics.disable();
        self
    }

    // ═══════════════════════════════════════════════════════════
    // ADMIN PANEL
    // ═══════════════════════════════════════════════════════════

    /// Configures and enables the `AdminPanel` via a closure.
    ///
    /// ```rust,ignore
    /// .with_admin(|a| a
    ///     .prefix("/admin")
    ///     .hot_reload(is_debug())
    ///     .site_title("My Admin")
    /// )
    /// ```
    pub fn with_admin(mut self, f: impl FnOnce(AdminStaging) -> AdminStaging) -> Self {
        self.admin = f(self.admin.enable());
        self
    }

    // ═══════════════════════════════════════════════════════════
    // RESET PASSWORD
    // ═══════════════════════════════════════════════════════════

    /// Enables the built-in password reset flow for a given entity.
    ///
    /// Automatically registers two routes:
    ///   - `{config.forgot_route}` — email form (step 1)
    ///   - `{config.reset_route}/{token}/{encrypted_email}` — new password (step 2)
    ///
    /// Minimal example (built-in entity):
    /// ```rust,ignore
    /// .with_password_reset::<BuiltinUserEntity>(|pr| pr)
    /// ```
    ///
    /// With custom config:
    /// ```rust,ignore
    /// .with_password_reset::<MyEntity>(|pr| pr
    ///     .forgot_route("/forgot-password")
    ///     .reset_route("/reset")
    ///     .base_url("https://mysite.com")
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
    // PHASE 2: VALIDATION + CONSTRUCTION (strict pipeline)
    //
    // Like Prisme (forms):
    // 1. `validate()` — checks each staging + cross-dependencies
    // 2. `all_ready()` — signal OK
    // 3. Construction in guaranteed STRICT order
    // 4. `MiddlewareStaging` reorganizes by slots and applies
    // ═══════════════════════════════════════════════════════════

    /// Validates and builds the application.
    ///
    /// # Construction Pipeline
    /// 1. **Validation** of all components (Core, Middleware, Statics)
    /// 2. **Construction** of the Core (Templates → Engine → URLs)
    /// 3. **Automatic reorganization** of middlewares by slots
    /// 4. **Application** of static files (if enabled)
    pub async fn build(mut self) -> Result<RuniqueApp, BuildError> {
        // ═══════════════════════════════════════
        // STEP 0: TRACING (before everything else)
        // ═══════════════════════════════════════
        self.config.log.init_subscriber();

        // ═══════════════════════════════════════
        // STEP 1: VALIDATION (like Prisme)
        // ═══════════════════════════════════════
        self.validate()?;

        if !self.all_ready() {
            return Err(BuildError::validation(
                "One or more components are not ready for construction",
            ));
        }

        // ═══════════════════════════════════════
        // STEP 2: DB CONNECTION (if DatabaseConfig provided)
        //
        // Two possible paths:
        //   1. `with_database(db)`        → already connected, we take as is
        //   2. `with_database_config(cfg)` → `connect()` during build
        // ═══════════════════════════════════════
        #[cfg(feature = "orm")]
        let db = self.core.connect().await?;

        // ═══════════════════════════════════════
        // STEP 3: DESTRUCTURING
        // ═══════════════════════════════════════
        let config = self.config;
        let url_registry = self.core.url_registry;
        let mut middleware = self.middleware;
        let statics_enabled = self.statics.enabled;
        let static_cache = self.statics.static_cache;
        let media_cache = self.statics.media_cache;
        let router = self.router;

        // ═══════════════════════════════════════
        // STEP 4: CORE CONSTRUCTION
        // Strict order: Templates → Config → Engine → URLs
        // ═══════════════════════════════════════

        // A. Templates (Tera) — always first
        let tera = new(TemplateLoader::init(&config, url_registry.clone())
            .map_err(|e| BuildError::template(e.to_string()))?);

        let config = new(config);
        log_init(config.log.clone());
        crate::utils::password::password_init(config.password.clone());

        // B. Engine (heart of the application)
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

        // C. URL registration (urlpatterns!)
        add_urls(&engine);

        // ═══════════════════════════════════════
        // ═══════════════════════════════════════
        // STEP 4b: ADMIN PANEL — merged BEFORE the middleware stack
        //
        // `.layer()` in Axum only covers routes already present
        // on the router at the time of the call.
        // Merging after = admin routes without Session/CSRF/Extensions.
        // ═══════════════════════════════════════

        let router = router.unwrap_or_default();

        // ═══════════════════════════════════════
        // STEP 4b.1: RESET PASSWORD (before middleware, like admin)
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
        // STEP 5: MIDDLEWARE STAGING
        //
        // Applied to all routes (dev + admin).
        // The staging automatically reorganizes by slots:
        //   Extensions → Session → CSRF → CSP → Host
        // ═══════════════════════════════════════

        let _exclusive_login = middleware.exclusive_login;
        let (router, session_store) =
            middleware.apply_to_router(router, config, engine.clone(), tera);
        if let Some(store) = session_store
            && let Ok(mut guard) = engine.session_store.write()
        {
            *guard = Some(store);
        }

        // Store DB sessions — initialized if a DB is available
        #[cfg(feature = "orm")]
        {
            let db_store = RuniqueSessionStore::new(engine.db.clone());
            if let Ok(mut guard) = engine.session_db_store.write() {
                *guard = Some(Arc::new(db_store));
            }
        }

        // ═══════════════════════════════════════
        // STEP 6: STATIC FILES (conditional)
        // ═══════════════════════════════════════

        let router = if statics_enabled {
            Self::attach_static_files(router, &engine.config, static_cache, media_cache)
        } else {
            router
        };

        Ok(RuniqueApp { engine, router })
    }

    // ═══════════════════════════════════════════════════════════
    // INTERNAL VALIDATION
    // ═══════════════════════════════════════════════════════════

    /// Individual validation of each staging, then cross-validation
    fn validate(&self) -> Result<(), BuildError> {
        // Individual validation (like `field.validate()` in Prisme)
        self.core.validate()?;
        self.middleware.validate()?;
        self.statics.validate()?;
        self.admin.validate()?;

        // Cross-validation (dependencies between components)
        self.cross_validate()?;

        Ok(())
    }

    /// Checks dependencies between components
    fn cross_validate(&self) -> Result<(), BuildError> {
        // Future inter-component validations:
        //
        // - host_validation enabled → ALLOWED_HOSTS defined?
        // - enable_debug_errors in production → warning
        // - CSP strict + session Memory → warning
        Ok(())
    }

    /// Checks that all components are ready
    fn all_ready(&self) -> bool {
        self.core.is_ready()
            && self.middleware.is_ready()
            && self.statics.is_ready()
            && self.admin.is_ready()
    }

    // ═══════════════════════════════════════════════════════════
    // STATIC FILES
    // ═══════════════════════════════════════════════════════════

    /// Attaches static file routes to the router
    fn attach_static_files(
        mut router: Router,
        config: &RuniqueConfig,
        static_cache: &'static str,
        media_cache: &'static str,
    ) -> Router {
        let security_headers = || {
            tower::ServiceBuilder::new()
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
                ))
        };

        let static_headers = security_headers().layer(SetResponseHeaderLayer::if_not_present(
            HeaderName::from_static("cache-control"),
            HeaderValue::from_static(static_cache),
        ));

        let media_headers = security_headers().layer(SetResponseHeaderLayer::if_not_present(
            HeaderName::from_static("cache-control"),
            HeaderValue::from_static(media_cache),
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
                media_headers.service(ServeDir::new(&config.static_files.media_root)),
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
