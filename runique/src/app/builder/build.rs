//! `build`: the order in which the application is put together. Each step is
//! a function of its own, in this file or beside it (`start`, `checks`,
//! `templates`, `routes`, `static_files`).

use std::sync::Arc;

use axum::Router;

use super::{
    super::runique_app::RuniqueApp,
    RuniqueAppBuilder, routes,
    start::{check_timezone, load_project_locales, start_logging},
    templates,
};
use crate::{
    app::staging::MiddlewareStaging,
    config::RuniqueConfig,
    engine::{RuniqueEngine, SecurityPolicies},
    errors::build::BuildError,
    middleware::{HostPolicy, MiddlewareConfig},
    router::add_urls,
    utils::aliases::{ADb, AEngine, new},
};

#[cfg(feature = "orm")]
use crate::middleware::session::session_db::RuniqueSessionStore;

impl<P> RuniqueAppBuilder<P> {
    /// Validates and builds the application.
    ///
    /// # Construction Pipeline
    /// 1. **Traces**, the project's translations, the time zone
    /// 2. **Validation** of all components (Core, Middleware, Statics, Admin)
    /// 3. **Database** connection
    /// 4. **Templates**, then the **engine** (config, Tera, security policies)
    /// 5. **Routes** Runique adds: welcome page, password reset, admin
    /// 6. **Middlewares**, sorted by slot, with the session stores created before the engine
    /// 7. **Static files**, the body size limit, the admin's `no-store`
    pub async fn build(mut self) -> Result<RuniqueApp, BuildError> {
        let log_guards = start_logging(&mut self.config);
        // The project's own translations, before anything is said in its
        // language: the checks below answer in it.
        load_project_locales(self.config.lang.as_deref())?;
        check_timezone(&self.config.timezone)?;

        self.validate()?;
        if !self.all_ready() {
            return Err(BuildError::validation(
                "One or more components are not ready for construction",
            ));
        }

        //   - `with_database(db)`        → already connected, taken as is
        //   - `with_database_config(cfg)` → `connect()` during build
        #[cfg(feature = "orm")]
        let db = self.core.connect().await?;

        #[cfg(feature = "admin")]
        let admin_cache_policy = routes::admin_cache_policy(&self.admin);
        #[cfg(feature = "admin")]
        let admin_enabled = self.admin.enabled;
        #[cfg(not(feature = "admin"))]
        let admin_enabled = false;

        let mut config = self.config;
        links_to_this_machine(&mut config);
        let url_registry = self.core.url_registry;
        let mut middleware = self.middleware;

        #[cfg(feature = "admin")]
        templates::write_admin_blocks(&self.admin, &mut config)?;
        #[cfg_attr(not(feature = "admin"), allow(unused_variables))]
        let (tera, integrity) = templates::load_templates(&config, &url_registry)?;
        #[cfg(feature = "admin")]
        templates::check_admin_pages(&self.admin, &config, &url_registry, &tera)?;

        let config = new(config);
        crate::security::password::password_init(config.password.clone());
        warn_hsts_preload(&config);
        warn_debug_in_prod(&config);

        #[cfg(feature = "orm")]
        let db = ADb::from_connection(db);
        // One database store, shared by the engine and the memory store's
        // fallback: it holds only the connection and the derived key.
        #[cfg(feature = "orm")]
        let database_sessions = Some(Arc::new(RuniqueSessionStore::new(
            db.clone(),
            &config.server.secret_key,
        )));
        #[cfg(not(feature = "orm"))]
        let database_sessions = None;
        let sessions = middleware.session_stores(database_sessions);

        let engine = new(RuniqueEngine {
            config: config.clone(),
            tera: new(tera),
            #[cfg(feature = "orm")]
            db,
            features: middleware_features(&middleware),
            url_registry,
            security: security_policies(&mut middleware, &config, admin_enabled),
            sessions,
            extensions: self.core.extensions,
        });
        add_urls(&engine);

        let router = self.router.unwrap_or_default();
        let router = routes::welcome_page(
            router,
            config.dev,
            self.root_free,
            self.password_reset.as_ref(),
        );
        #[cfg(feature = "admin")]
        let admin_reset = routes::admin_reset_page(&mut self.admin, self.password_reset.as_ref());
        #[cfg(not(feature = "admin"))]
        let admin_reset = None;
        let router =
            routes::router_reset_password(router, &engine, self.password_reset, admin_reset);
        #[cfg(feature = "admin")]
        let router = routes::router_admin(router, &engine, self.admin, &integrity).await?;
        routes::log_url_registry(&engine);

        let router = middleware.apply_middleware(router, engine.clone())?;

        let router = if self.statics.enabled {
            Self::attach_static_files(
                router,
                &engine.config,
                self.statics.static_cache,
                self.statics.media_cache,
            )
        } else {
            router
        };
        let router = limit_body(router, &engine);
        #[cfg(feature = "admin")]
        let router = routes::no_store_on_admin(router, admin_cache_policy);

        Ok(RuniqueApp {
            engine,
            router,
            _log_guards: log_guards,
        })
    }
}

/// In development, links and assets must lead back to this machine, never
/// to the production site a hard-coded `.with_public_url()` names.
fn links_to_this_machine(config: &mut RuniqueConfig) {
    if config.dev {
        config.server.public_url = Some(format!("http://127.0.0.1:{}", config.server.port));
    }
}

/// HSTS `preload` requires `includeSubDomains` + `max-age ≥ 1 an` to be
/// accepted on the browser preload list — warn once at boot if the combo
/// is invalid (the header would otherwise be silently ignored for preload).
fn warn_hsts_preload(config: &RuniqueConfig) {
    if config.security.hsts_preload_misconfigured() {
        tracing::warn!(
            max_age = config.security.hsts_max_age,
            include_subdomains = config.security.hsts_include_subdomains,
            "HSTS_PRELOAD set but combo invalid (needs includeSubDomains + max-age ≥ 31536000) — ignored by browsers for preload"
        );
    }
}

fn warn_debug_in_prod(config: &RuniqueConfig) {
    if config.debug && !config.dev {
        tracing::warn!(
            "{}",
            crate::translation::t(runique_i18n::keys::build::log::DEBUG_IN_PROD)
        );
    }
}

/// The middleware toggles the engine keeps, with the exclusive login the
/// admin reads at sign-in.
fn middleware_features(middleware: &MiddlewareStaging) -> MiddlewareConfig {
    let mut features = middleware.features.clone();
    features.exclusive_login = middleware.exclusive_login;
    features
}

fn security_policies(
    middleware: &mut MiddlewareStaging,
    config: &RuniqueConfig,
    admin_enabled: bool,
) -> SecurityPolicies {
    let mut csp = middleware.security_policy.take().unwrap_or_default();
    // `strict()` turns it on, and a local server only speaks HTTP.
    if config.dev {
        csp.upgrade_insecure_requests = false;
    }
    // Only the admin adds to the policy.
    if admin_enabled {
        csp.merge_htmx_hashes();
    }
    SecurityPolicies {
        csp: new(csp),
        hosts: new(HostPolicy::new(
            middleware.allowed_hosts.clone(),
            middleware.features.enable_host_validation,
        )),
        csrf_exempt_paths: Arc::new(middleware.csrf_exempt_paths.clone()),
        permissions_policy: Arc::new(middleware.permissions_policy.take().unwrap_or_default()),
        trusted_proxies: Arc::new(match middleware.trusted_proxies_config.take() {
            // Explicit config always wins; otherwise the default is edge-aware
            // (private ranges behind a proxy, but `none()` when Runique is the
            // ACME TLS edge — closes SEC2). See `TrustedProxies::default_for_edge`.
            Some(c) => c.build(),
            None => crate::middleware::security::trusted_proxies::TrustedProxies::default_for_edge(
                config.security.acme_enabled,
            ),
        }),
    }
}

/// Largest request body: the upload limit, plus 1 MB for the form's text fields.
fn body_limit(max_upload_mb: u64) -> usize {
    let bytes = max_upload_mb.saturating_add(1).saturating_mul(1024 * 1024);
    usize::try_from(bytes).unwrap_or(usize::MAX)
}

/// A multipart body streams to disk, each file held to `max_upload_mb` by
/// `parse_multipart`, the whole request to `max_body_mb`. Bodies read into
/// memory (urlencoded, JSON) are held to `FORM_BODY_LIMIT` by Aegis.
fn limit_body(router: Router, engine: &AEngine) -> Router {
    router.layer(axum::extract::DefaultBodyLimit::max(body_limit(
        engine.config.static_files.max_body_mb,
    )))
}
