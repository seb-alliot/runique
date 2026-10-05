//! RuniqueAppBuilder — collection phase: stores configuration without executing it.
mod build;

use axum::Router;
use std::marker::PhantomData;
use tower_sessions::cookie::time::Duration;

use super::staging::{AdminStaging, CoreStaging, MiddlewareStaging, StaticStaging};
use crate::auth::{PasswordResetConfig, PasswordResetStaging};
use crate::config::RuniqueConfig;
use crate::utils::runique_log::RuniqueLog;

#[cfg(feature = "orm")]
use crate::db::DatabaseConfig;
#[cfg(feature = "orm")]
use sea_orm::DatabaseConnection;

/// Which once-only settings the builder already holds, carried in its type:
/// calling one of them a second time doesn't compile, rather than silently
/// replacing (or, for the mailer, silently ignoring) the first call.
///
/// The state is a tuple with one slot per setting — public URL, routes, logs,
/// password reset, database, session duration, mailer — each [`No`] until
/// set, [`Yes`] after. Settings that compose (`core`, `middleware`,
/// `static_files`, `with_admin`, `with_custom_db`, `statics`) take no slot.
///
/// [`No`]: state::No
/// [`Yes`]: state::Yes
pub mod state {
    /// The setting isn't set yet.
    pub struct No;
    /// The setting is set: it can't be set again.
    pub struct Yes;

    /// A builder that holds none of the once-only settings yet.
    pub type Fresh = (No, No, No, No, No, No, No);

    mod sealed {
        pub trait Sealed {}
        impl Sealed for super::No {}
    }

    macro_rules! once_only {
        ($(#[$doc:meta])* $name:ident, $message:literal, $label:literal, $note:literal) => {
            $(#[$doc])*
            #[diagnostic::on_unimplemented(message = $message, label = $label, note = $note)]
            pub trait $name: sealed::Sealed {}
            impl $name for No {}
        };
    }

    once_only!(
        /// `with_public_url()` not called yet.
        PublicUrlUnset,
        "`with_public_url()` has already been called on this builder",
        "a second `with_public_url()` would silently replace the first",
        "keep a single `.with_public_url(...)`; to tell local from production apart, wrap it in an `if`"
    );
    once_only!(
        /// `routes()` not called yet.
        RoutesUnset,
        "`routes()` has already been called on this builder",
        "a second `routes()` would replace every route of the first",
        "build a single `Router` (merge or nest the others into it) and pass it once"
    );
    once_only!(
        /// `with_log()` not called yet.
        LogUnset,
        "`with_log()` has already been called on this builder",
        "a second `with_log()` would start again from an empty log configuration",
        "set every category in a single `.with_log(|l| ...)`"
    );
    once_only!(
        /// `with_password_reset()` not called yet.
        PasswordResetUnset,
        "`with_password_reset()` has already been called on this builder",
        "a second `with_password_reset()` would start again from the default configuration",
        "configure everything in a single `.with_password_reset(|pr| ...)`"
    );
    once_only!(
        /// Neither `with_database()` nor `with_database_config()` called yet.
        DatabaseUnset,
        "the database has already been set on this builder",
        "`with_database()` and `with_database_config()` set the same database: a second call would replace the first",
        "pass a single connection or a single configuration"
    );
    once_only!(
        /// `with_session_duration()` not called yet.
        SessionDurationUnset,
        "`with_session_duration()` has already been called on this builder",
        "a second `with_session_duration()` would silently replace the first",
        "keep a single `.with_session_duration(...)`"
    );
    once_only!(
        /// Neither `with_mailer()` nor `with_mailer_from_env()` called yet.
        MailerUnset,
        "the mailer has already been set on this builder",
        "`with_mailer()` and `with_mailer_from_env()` set the same mailer: the second call would be ignored",
        "keep a single mailer call"
    );
}

use state::{Fresh, Yes};

/// Intelligent application builder for Runique
///
#[doc = include_str!("../../../doc-tests/builder/builder_basic.md")]
pub struct RuniqueAppBuilder<S = Fresh> {
    pub(super) config: RuniqueConfig,
    pub(super) core: CoreStaging,
    pub(super) middleware: MiddlewareStaging,
    pub(super) statics: StaticStaging,
    pub(super) router: Option<Router>,
    pub(super) admin: AdminStaging,
    pub(super) password_reset: Option<PasswordResetStaging>,
    pub(super) state: PhantomData<S>,
}

impl RuniqueAppBuilder<Fresh> {
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
            state: PhantomData,
        }
    }
}

impl<S> RuniqueAppBuilder<S> {
    /// The same builder, holding the state `T`.
    fn retype<T>(self) -> RuniqueAppBuilder<T> {
        RuniqueAppBuilder {
            config: self.config,
            core: self.core,
            middleware: self.middleware,
            statics: self.statics,
            router: self.router,
            admin: self.admin,
            password_reset: self.password_reset,
            state: PhantomData,
        }
    }
}

// ═══════════════════════════════════════════════════════════
// ONCE-ONLY SETTINGS
//
// Each one fills its slot of the state: a second call doesn't compile.
// ═══════════════════════════════════════════════════════════

impl<PU, RT, LG, PR, DB, SD, ML> RuniqueAppBuilder<(PU, RT, LG, PR, DB, SD, ML)> {
    /// Sets the public URL of the application (`https://mysite.com`): the
    /// address a visitor types, base of every absolute link the app sends out
    /// — password reset links among them — and of the admin's "view site"
    /// link unless the admin sets its own. Required in production when the
    /// password reset or the admin is enabled; in debug, the request's `Host`
    /// stands in for it.
    ///
    /// Called once: a second call doesn't compile, rather than silently
    /// replacing the first.
    ///
    /// ```
    /// # use runique::{app::RuniqueApp, config::RuniqueConfig};
    /// let builder = RuniqueApp::builder(RuniqueConfig::default())
    ///     .with_public_url("https://mysite.com");
    /// ```
    ///
    /// ```compile_fail,E0277
    /// # use runique::{app::RuniqueApp, config::RuniqueConfig};
    /// let builder = RuniqueApp::builder(RuniqueConfig::default())
    ///     .with_public_url("http://localhost:3000")
    ///     .with_public_url("https://mysite.com");
    /// ```
    pub fn with_public_url(mut self, url: &str) -> RuniqueAppBuilder<(Yes, RT, LG, PR, DB, SD, ML)>
    where
        PU: state::PublicUrlUnset,
    {
        let url = url.trim().trim_end_matches('/');
        self.config.server.public_url = (!url.is_empty()).then(|| url.to_string());
        self.retype()
    }

    /// Defines the application routes, once: build a single `Router` (merge or
    /// nest the others into it) — a second call doesn't compile.
    ///
    /// ```compile_fail,E0277
    /// # use runique::{app::RuniqueApp, config::RuniqueConfig};
    /// let builder = RuniqueApp::builder(RuniqueConfig::default())
    ///     .routes(axum::Router::new())
    ///     .routes(axum::Router::new());
    /// ```
    pub fn routes(mut self, router: Router) -> RuniqueAppBuilder<(PU, Yes, LG, PR, DB, SD, ML)>
    where
        RT: state::RoutesUnset,
    {
        self.router = Some(router);
        self.retype()
    }

    /// Configures Runique logs by category, once: a second call doesn't compile.
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
    pub fn with_log(
        mut self,
        f: impl FnOnce(RuniqueLog) -> RuniqueLog,
    ) -> RuniqueAppBuilder<(PU, RT, Yes, PR, DB, SD, ML)>
    where
        LG: state::LogUnset,
    {
        self.config.log = f(RuniqueLog::new());
        self.retype()
    }

    /// Enables the built-in password reset flow (accounts of `eihwaz_users`),
    /// once: a second call doesn't compile.
    ///
    /// Automatically registers two routes:
    ///   - `{config.forgot_route}` — email form (step 1)
    ///   - `{config.reset_route}/{token}/{encrypted_email}` — new password (step 2)
    ///
    /// Minimal example (built-in entity):
    /// ```rust,ignore
    /// .with_password_reset(|pr| pr)
    /// ```
    ///
    /// With custom config:
    /// ```rust,ignore
    /// .with_password_reset(|pr| pr
    ///     .forgot_route("/forgot-password")
    ///     .reset_route("/reset")
    /// )
    /// ```
    pub fn with_password_reset(
        mut self,
        f: impl FnOnce(PasswordResetConfig) -> PasswordResetConfig,
    ) -> RuniqueAppBuilder<(PU, RT, LG, Yes, DB, SD, ML)>
    where
        PR: state::PasswordResetUnset,
    {
        let config = f(PasswordResetConfig::default());
        self.password_reset = Some(PasswordResetStaging { config });
        self.retype()
    }

    /// Shortcut: adds an already established DB connection without going
    /// through `.core()`. The database is set once, by this method or
    /// [`with_database_config`](Self::with_database_config): a second call
    /// doesn't compile.
    ///
    /// ```rust,ignore
    /// let db = DatabaseConfig::from_env()?.build().connect().await?;
    /// RuniqueApp::builder(config).with_database(db)
    /// ```
    ///
    /// ```compile_fail,E0277
    /// # use runique::{app::RuniqueApp, config::RuniqueConfig, db::DatabaseConfig};
    /// # fn conn() -> runique::sea_orm::DatabaseConnection { unimplemented!() }
    /// # fn cfg() -> DatabaseConfig { unimplemented!() }
    /// let builder = RuniqueApp::builder(RuniqueConfig::default())
    ///     .with_database(conn())
    ///     .with_database_config(cfg());
    /// ```
    #[cfg(feature = "orm")]
    pub fn with_database(
        mut self,
        db: DatabaseConnection,
    ) -> RuniqueAppBuilder<(PU, RT, LG, PR, Yes, SD, ML)>
    where
        DB: state::DatabaseUnset,
    {
        self.core = self.core.with_database(db);
        self.retype()
    }

    /// Shortcut: adds a DB configuration — auto-connection during `build()`.
    /// The database is set once, by this method or
    /// [`with_database`](Self::with_database).
    ///
    /// ```rust,ignore
    /// let db_config = DatabaseConfig::from_env()?.build();
    /// RuniqueApp::builder(config).with_database_config(db_config)
    /// ```
    #[cfg(feature = "orm")]
    pub fn with_database_config(
        mut self,
        config: DatabaseConfig,
    ) -> RuniqueAppBuilder<(PU, RT, LG, PR, Yes, SD, ML)>
    where
        DB: state::DatabaseUnset,
    {
        self.core = self.core.with_database_config(config);
        self.retype()
    }

    /// Duration of an **authenticated** session (Django's `SESSION_COOKIE_AGE` equivalent),
    /// set once: a second call doesn't compile.
    ///
    /// Single source, settable **only here** (no `.env`): applies to the cookie, to the
    /// `eihwaz_sessions` row (`expires_at`), and to the per-request refresh — `login()`
    /// reads the same value, so nothing can drift. Default when not called: **24h**.
    /// Anonymous sessions:
    /// [`with_anonymous_session_duration`](MiddlewareStaging::with_anonymous_session_duration).
    /// Full docs (including "remember me"): `docs/*/middleware/sessions`.
    pub fn with_session_duration(
        mut self,
        duration: Duration,
    ) -> RuniqueAppBuilder<(PU, RT, LG, PR, DB, Yes, ML)>
    where
        SD: state::SessionDurationUnset,
    {
        self.middleware = self.middleware.with_session_duration(duration);
        self.retype()
    }

    /// Configures the SMTP mailer manually. The mailer is set once, by this
    /// method or [`with_mailer_from_env`](Self::with_mailer_from_env): a second
    /// call doesn't compile.
    ///
    /// ```rust,ignore
    /// builder::new(config)
    ///     .with_mailer(MailerConfig { host: "smtp.example.com".into(), port: 587, ... })
    /// ```
    ///
    /// ```compile_fail,E0277
    /// # use runique::{app::RuniqueApp, config::RuniqueConfig};
    /// # fn mailer() -> runique::utils::mailer::MailerConfig { unimplemented!() }
    /// let builder = RuniqueApp::builder(RuniqueConfig::default())
    ///     .with_mailer(mailer())
    ///     .with_mailer_from_env();
    /// ```
    pub fn with_mailer(
        self,
        config: crate::utils::mailer::MailerConfig,
    ) -> RuniqueAppBuilder<(PU, RT, LG, PR, DB, SD, Yes)>
    where
        ML: state::MailerUnset,
    {
        crate::utils::mailer::mailer_init(config);
        self.retype()
    }

    /// Configures the mailer from environment variables
    /// (SMTP_HOST, SMTP_USER, SMTP_PASS, SMTP_FROM, SMTP_PORT, SMTP_STARTTLS).
    /// The mailer is set once, by this method or [`with_mailer`](Self::with_mailer).
    pub fn with_mailer_from_env(self) -> RuniqueAppBuilder<(PU, RT, LG, PR, DB, SD, Yes)>
    where
        ML: state::MailerUnset,
    {
        crate::utils::mailer::mailer_init_from_env();
        self.retype()
    }
}

// ═══════════════════════════════════════════════════════════
// SETTINGS THAT COMPOSE
//
// A second call continues from the first: no slot.
// ═══════════════════════════════════════════════════════════

impl<S> RuniqueAppBuilder<S> {
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

    /// Shortcut: registers a custom external database (MongoDB, Redis, etc.).
    ///
    /// ```rust,ignore
    /// let mongo = mongodb::Client::with_options(opts)?.into();
    /// RuniqueApp::builder(config).with_custom_db(mongo)
    /// ```
    pub fn with_custom_db<T: std::any::Any + Send + Sync + 'static>(mut self, db: T) -> Self {
        self.core = self.core.with_extra_db(db);
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

    // ─── Static files ─────────────────────────────────────────────────────────

    /// Configures static files via a closure.
    ///
    /// # Example
    /// ```rust,ignore
    /// .static_files(|s| s.enabled(false))
    /// ```
    pub fn static_files(mut self, f: impl FnOnce(StaticStaging) -> StaticStaging) -> Self {
        self.statics = f(self.statics);
        self
    }

    /// Shortcut: enables the static files service (enabled by default).
    pub fn statics(mut self) -> Self {
        self.statics = self.statics.enabled(true);
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
}
