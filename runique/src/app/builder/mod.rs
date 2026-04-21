//! RuniqueAppBuilder — collection phase: stores configuration without executing it.
mod build;

use axum::Router;
use tower_sessions::cookie::time::Duration;

use super::staging::{AdminStaging, CoreStaging, MiddlewareStaging, StaticStaging};
use crate::auth::{
    PasswordResetAdapter, PasswordResetConfig, PasswordResetStaging, session::UserEntity,
};
use crate::config::RuniqueConfig;
use crate::utils::runique_log::RuniqueLog;

#[cfg(feature = "orm")]
use crate::db::DatabaseConfig;
#[cfg(feature = "orm")]
use sea_orm::DatabaseConnection;

/// Intelligent application builder for Runique
///
#[doc = include_str!("../../../doc-tests/builder/builder_basic.md")]
pub struct RuniqueAppBuilder {
    pub(super) config: RuniqueConfig,
    pub(super) core: CoreStaging,
    pub(super) middleware: MiddlewareStaging,
    pub(super) statics: StaticStaging,
    pub(super) router: Option<Router>,
    pub(super) admin: AdminStaging,
    pub(super) password_reset: Option<PasswordResetStaging>,
}

impl RuniqueAppBuilder {
    /// Creates a new intelligent builder with the given configuration.
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

    // ═══════════════════════════════════════════════════════════
    // PHASE 1: FLEXIBLE COLLECTION
    //
    // Each method stores the data without executing it.
    // Regardless of the call order by a dev.
    // ═══════════════════════════════════════════════════════════

    // ─── Core ────────────────────────────────────────────────────────────────

    /// Configures the core via a closure.
    ///
    /// # Example
    /// ```rust,ignore
    /// .core(|c| c.with_database(db))
    /// ```
    pub fn core(mut self, f: impl FnOnce(CoreStaging) -> CoreStaging) -> Self {
        self.core = f(self.core);
        self
    }

    /// Shortcut: adds an already established DB connection without going through `.core()`.
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

    /// Shortcut: adds a DB configuration — auto-connection during `build()`.
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

    // ─── Routes ──────────────────────────────────────────────────────────────

    /// Defines the application routes.
    pub fn routes(mut self, router: Router) -> Self {
        self.router = Some(router);
        self
    }

    // ─── Middleware ───────────────────────────────────────────────────────────

    /// Configures middlewares via a closure.
    ///
    /// The order of calls inside the closure does not matter:
    /// the framework will apply middlewares in the optimal guaranteed order
    /// thanks to the slots system.
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

    /// Shortcut: configures the session duration without going through `.middleware()`.
    pub fn with_session_duration(mut self, duration: Duration) -> Self {
        self.middleware = self.middleware.with_session_duration(duration);
        self
    }

    /// Shortcut: enables/disables debug error pages.
    pub fn with_error_handler(mut self, enable: bool) -> Self {
        self.middleware = self.middleware.with_debug_errors(enable);
        self
    }

    // ─── Static files ─────────────────────────────────────────────────────────

    /// Configures static files via a closure.
    ///
    /// # Example
    /// ```rust,ignore
    /// .static_files(|s| s.disable())
    /// ```
    pub fn static_files(mut self, f: impl FnOnce(StaticStaging) -> StaticStaging) -> Self {
        self.statics = f(self.statics);
        self
    }

    /// Configures the SMTP mailer manually.
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
    /// (SMTP_HOST, SMTP_USER, SMTP_PASS, SMTP_FROM, SMTP_PORT, SMTP_STARTTLS).
    pub fn with_mailer_from_env(self) -> Self {
        crate::utils::mailer::mailer_init_from_env();
        self
    }

    /// Shortcut: enables the static files service (enabled by default).
    pub fn statics(mut self) -> Self {
        self.statics = self.statics.enable();
        self
    }

    /// Shortcut: disables the static files service.
    pub fn no_statics(mut self) -> Self {
        self.statics = self.statics.disable();
        self
    }

    // ─── Admin panel ──────────────────────────────────────────────────────────

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

    // ─── Password reset ───────────────────────────────────────────────────────

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
}
