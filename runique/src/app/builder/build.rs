//! Build pipeline: validation → core construction → middleware → statics.
//!
//! Construction order:
//!   1. Validation (each staging + cross-dependencies)
//!   2. DB connection (if DatabaseConfig provided)
//!   3. Core (Templates → Engine → URLs)
//!   4. Admin panel (merged before middleware stack)
//!   5. Password reset routes
//!   6. Middleware staging (slots sort + apply)
//!   7. Static files

use std::sync::Arc;

use axum::{
    Router,
    http::{HeaderName, HeaderValue},
};
use tower_http::{services::ServeDir, set_header::SetResponseHeaderLayer};

use super::super::error_build::BuildError;
use super::super::runique_app::RuniqueApp;
use super::super::templates::TemplateLoader;
use super::RuniqueAppBuilder;
use crate::admin::build_admin_router;
use crate::config::RuniqueConfig;
use crate::engine::RuniqueEngine;
use crate::macros::{add_urls, register_name_url};
use crate::middleware::HostPolicy;
use crate::utils::aliases::new;
use crate::utils::runique_log::log_init;

#[cfg(feature = "orm")]
use crate::middleware::session::session_db::RuniqueSessionStore;

impl RuniqueAppBuilder {
    /// Validates and builds the application.
    ///
    /// # Construction Pipeline
    /// 1. **Validation** of all components (Core, Middleware, Statics)
    /// 2. **Construction** of the Core (Templates → Engine → URLs)
    /// 3. **Automatic reorganization** of middlewares by slots
    /// 4. **Application** of static files (if enabled)
    pub async fn build(mut self) -> Result<RuniqueApp, BuildError> {
        // Step 0: tracing (before everything else)
        // log_init early so get_log() works in TemplateLoader::init() and middleware staging.
        log_init(self.config.log.clone());
        let log_guards = self.config.log.init_subscriber();

        // Step 1: validation
        self.validate()?;
        if !self.all_ready() {
            return Err(BuildError::validation(
                "One or more components are not ready for construction",
            ));
        }

        // Step 2: DB connection (if DatabaseConfig provided)
        //   - `with_database(db)`        → already connected, taken as is
        //   - `with_database_config(cfg)` → `connect()` during build
        #[cfg(feature = "orm")]
        let db = self.core.connect().await?;

        // Step 3: destructuring
        let extensions = self.core.extensions;
        let config = self.config;
        let url_registry = self.core.url_registry;
        let mut middleware = self.middleware;
        let statics_enabled = self.statics.enabled;
        let static_cache = self.statics.static_cache;
        let media_cache = self.statics.media_cache;
        let router = self.router;

        // Step 4: core construction — strict order: Templates → Config → Engine → URLs

        let tera = new(TemplateLoader::init(&config, url_registry.clone())
            .map_err(|e| BuildError::template(e.to_string()))?);

        let config = new(config);
        crate::utils::password::password_init(config.password.clone());

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
            csrf_exempt_paths: Arc::new(middleware.csrf_exempt_paths.clone()),
            permissions_policy: Arc::new(middleware.permissions_policy.take().unwrap_or_default()),
            trusted_proxies: Arc::new(
                middleware
                    .trusted_proxies_config
                    .take()
                    .map(|c| c.build())
                    .unwrap_or_default(),
            ),
            session_store: std::sync::LazyLock::new(|| std::sync::RwLock::new(None)),
            session_db_store: std::sync::LazyLock::new(|| std::sync::RwLock::new(None)),
            extensions,
        });

        add_urls(&engine);

        // Step 4b: admin + password reset — merged BEFORE the middleware stack.
        // `.layer()` in Axum only covers routes present at call time;
        // merging after means admin routes run without Session/CSRF/Extensions.
        let router = router.unwrap_or_default();

        let router = if let Some(pr) = self.password_reset {
            let forgot_path = pr.config.forgot_route.clone();
            let reset_path = format!(
                "{}/{{token}}/{{encrypted_email}}",
                pr.config.reset_route.trim_end_matches('/')
            );
            let pr_router = pr.handler.build_router(Arc::new(pr.config));
            register_name_url(&engine, "forgot_password", &forgot_path);
            register_name_url(&engine, "reset_password", &reset_path);
            router.merge(pr_router)
        } else {
            router
        };

        let router = if self.admin.enabled {
            let admin_prefix = self.admin.config.prefix.trim_end_matches('/').to_string();
            let robots_txt = self.admin.robots_txt;
            let sitemap_url = self.admin.sitemap_url.clone();
            if let Some(level) = crate::utils::runique_log::get_log()
                .builder
                .as_ref()
                .and_then(|b| b.registry)
            {
                let count = self
                    .admin
                    .state
                    .as_ref()
                    .map(|s| s.registry.all().count())
                    .unwrap_or(0);
                crate::runique_log!(level, resources = count, "admin registry");
            }
            let admin_router = build_admin_router(self.admin, engine.db.clone());
            add_urls(&engine);
            register_name_url(&engine, "admin", &admin_prefix);
            let mut r = router.merge(admin_router);
            if robots_txt {
                r = r.route(
                    "/robots.txt",
                    axum::routing::get(move || {
                        let sitemap_line = sitemap_url
                            .map(|u| format!("Sitemap: {}\n", u))
                            .unwrap_or_default();
                        let body = format!(
                            "User-agent: *\nDisallow: {}/\n\n{}Content-Signal: ai-train=yes, search=yes, ai-input=yes\n",
                            admin_prefix, sitemap_line
                        );
                        async move { body }
                    }),
                );
            }
            r
        } else {
            router
        };

        if let Some(level) = crate::utils::runique_log::get_log()
            .builder
            .as_ref()
            .and_then(|b| b.routes)
        {
            let count = engine
                .url_registry
                .read()
                .map(|guard| guard.len())
                .unwrap_or(0);
            crate::runique_log!(level, routes = count, "url registry");
        }

        // Step 5: middleware staging — automatic slot sort and apply
        let _exclusive_login = middleware.exclusive_login;
        let (router, session_store) =
            middleware.apply_to_router(router, config, engine.clone(), tera);
        if let Some(store) = session_store
            && let Ok(mut guard) = engine.session_store.write()
        {
            *guard = Some(store);
        }

        #[cfg(feature = "orm")]
        {
            let db_store = RuniqueSessionStore::new(engine.db.clone());
            if let Ok(mut guard) = engine.session_db_store.write() {
                *guard = Some(Arc::new(db_store));
            }
        }

        // Step 6: static files (conditional)
        let router = if statics_enabled {
            Self::attach_static_files(router, &engine.config, static_cache, media_cache)
        } else {
            router
        };

        Ok(RuniqueApp {
            engine,
            router,
            _log_guards: log_guards,
        })
    }

    // ─── Internal validation ──────────────────────────────────────────────────

    fn validate(&self) -> Result<(), BuildError> {
        self.core.validate()?;
        self.middleware.validate()?;
        self.statics.validate()?;
        self.admin.validate()?;
        self.cross_validate()
    }

    fn cross_validate(&self) -> Result<(), BuildError> {
        if self.config.debug {
            return Ok(());
        }

        use crate::app::error_build::{CheckError, CheckReport};

        let mut report = CheckReport::new();
        #[cfg(feature = "acme")]
        let sec = &self.config.security;
        let srv = &self.config.server;

        if srv.secret_key == "default_secret_key" {
            report.add(
                CheckError::new("Security", "SECRET_KEY is using the default insecure value")
                    .with_suggestion(
                        "Set SECRET_KEY to a random 32+ character string in your .env file",
                    ),
            );
        }

        #[cfg(feature = "acme")]
        if sec.acme_enabled {
            if sec.acme_domain.is_none() {
                report.add(
                    CheckError::new("ACME", "ACME_ENABLED=true but ACME_DOMAIN is not set")
                        .with_suggestion(
                            "Set ACME_DOMAIN to your production domain in your .env file",
                        ),
                );
            }
            if sec.acme_email.is_none() {
                report.add(
                    CheckError::new("ACME", "ACME_ENABLED=true but ACME_EMAIL is not set")
                        .with_suggestion(
                            "Set ACME_EMAIL to your Let's Encrypt contact email in your .env file",
                        ),
                );
            }
        }

        if report.has_errors() {
            return Err(BuildError::check(report));
        }

        Ok(())
    }

    fn all_ready(&self) -> bool {
        self.core.is_ready()
            && self.middleware.is_ready()
            && self.statics.is_ready()
            && self.admin.is_ready()
    }

    // ─── Static files attachment ──────────────────────────────────────────────

    pub(super) fn attach_static_files(
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

        if let Some(level) = crate::utils::runique_log::get_log()
            .builder
            .as_ref()
            .and_then(|b| b.statics)
        {
            crate::runique_log!(
                level,
                static_url = %config.static_files.static_url,
                static_dir = %config.static_files.staticfiles_dirs,
                media_url = %config.static_files.media_url,
                media_dir = %config.static_files.media_root,
                "static files attached"
            );
        }

        router
    }
}
