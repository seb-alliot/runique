//! Centralized Runique log configuration — a tree of per-module tracing categories,
//! the `runique_log!` macro, and the global subscriber init.
//!
//! The configuration mirrors the framework module map: one sub-struct per subsystem
//! (`forms`, `middleware`, `session`, `auth`, `admin`, `db`, `mailer`, `migration`,
//! `templates`, `errors`, `builder`). Each leaf is an `Option<Level>` — `None` means
//! the event is disabled (zero cost).
use std::sync::{Arc, Mutex};
use tracing_subscriber::{EnvFilter, fmt::format::FmtSpan};

mod admin;
mod auth;
mod builder;
mod db;
mod errors;
mod forms;
mod mailer;
mod middleware;
mod migration;
mod session;
mod templates;

pub use admin::AdminTracing;
pub use auth::AuthTracing;
pub use builder::BuilderTracing;
pub use db::DbTracing;
pub use errors::ErrorsTracing;
pub use forms::FormTracing;
pub use mailer::MailerTracing;
pub use middleware::MiddlewareTracing;
pub use migration::MigrationTracing;
pub use session::SessionTracing;
pub use templates::TemplatesTracing;

/// Unified Runique log configuration — a tree of per-module categories.
///
/// # Exemple
/// ```rust,ignore
/// RuniqueApp::builder(config)
///     .with_log(|l| l
///         .subscriber_level("info")              // global default — RUST_LOG always wins
///         .middleware(|m| m.csrf(Level::WARN).host_validation(Level::INFO))
///         .session(|s| s.store(Level::INFO))
///         .forms(|f| f.validate(Level::DEBUG))
///         .admin(|a| a.crud(Level::INFO))
///     )
/// ```
#[derive(Debug, Clone, Default)]
pub struct RuniqueLog {
    /// Tracing subscriber level. `RUST_LOG` takes priority if defined.
    /// Default: `"debug"` if `DEBUG=true`, else `"warn"`.
    subscriber_level: Option<String>,

    /// Form pipeline tracing (`#[form]` fields, validation, render, finalize).
    pub forms: Option<FormTracing>,
    /// Middleware/security tracing (csrf, csp, cors, rate_limit, host_validation, …).
    pub middleware: Option<MiddlewareTracing>,
    /// Session lifecycle tracing (store, cleanup, exclusive_login).
    pub session: Option<SessionTracing>,
    /// Auth tracing (login, reset, password_init).
    pub auth: Option<AuthTracing>,
    /// Admin panel tracing (auth, crud, list, bulk, filter_fn, roles, daemon).
    pub admin: Option<AdminTracing>,
    /// Database tracing (connect, query).
    pub db: Option<DbTracing>,
    /// Mailer tracing (email dispatch results).
    pub mailer: Option<MailerTracing>,
    /// Migration / makemigrations tracing (plan, apply, rollback).
    pub migration: Option<MigrationTracing>,
    /// Template engine tracing (load, render).
    pub templates: Option<TemplatesTracing>,
    /// HTTP error-handling tracing (http, render).
    pub errors: Option<ErrorsTracing>,
    /// Builder startup tracing (templates, registry, middleware slots).
    pub builder: Option<BuilderTracing>,
}

impl RuniqueLog {
    pub fn new() -> Self {
        Self::default()
    }

    /// Overrides the tracing subscriber level.
    /// `RUST_LOG` always has priority over this value.
    #[must_use]
    pub fn subscriber_level(mut self, level: impl Into<String>) -> Self {
        self.subscriber_level = Some(level.into());
        self
    }

    /// Initializes the global tracing subscriber.
    /// Called automatically by `build()` — no effect if already initialized.
    pub fn init_subscriber(&self) {
        let default = self.subscriber_level.as_deref().unwrap_or_else(|| {
            if crate::utils::env::is_debug() {
                "debug"
            } else {
                "warn"
            }
        });

        let filter =
            std::env::var("RUST_LOG").map_or_else(|_| EnvFilter::new(default), EnvFilter::new);

        let already_installed = tracing_subscriber::fmt()
            .with_env_filter(filter)
            .with_span_events(FmtSpan::CLOSE)
            .try_init()
            .is_err();

        // A global tracing subscriber can only be set once per process, and the library
        // forbids replacing it. If the host app already installed one, Runique's config
        // is inactive — surface it instead of failing silently (the warning is captured
        // by the already-installed subscriber).
        if already_installed {
            tracing::warn!(
                "Runique: a tracing subscriber is already installed; Runique log configuration is inactive (RUST_LOG / with_log have no effect)"
            );
        }
    }

    /// Configures form pipeline tracing.
    #[must_use]
    pub fn forms(mut self, f: impl FnOnce(FormTracing) -> FormTracing) -> Self {
        self.forms = Some(f(self.forms.take().unwrap_or_default()));
        self
    }

    /// Configures middleware/security tracing.
    #[must_use]
    pub fn middleware(mut self, f: impl FnOnce(MiddlewareTracing) -> MiddlewareTracing) -> Self {
        self.middleware = Some(f(self.middleware.take().unwrap_or_default()));
        self
    }

    /// Configures session lifecycle tracing.
    #[must_use]
    pub fn session(mut self, f: impl FnOnce(SessionTracing) -> SessionTracing) -> Self {
        self.session = Some(f(self.session.take().unwrap_or_default()));
        self
    }

    /// Configures auth tracing.
    #[must_use]
    pub fn auth(mut self, f: impl FnOnce(AuthTracing) -> AuthTracing) -> Self {
        self.auth = Some(f(self.auth.take().unwrap_or_default()));
        self
    }

    /// Configures admin panel tracing.
    #[must_use]
    pub fn admin(mut self, f: impl FnOnce(AdminTracing) -> AdminTracing) -> Self {
        self.admin = Some(f(self.admin.take().unwrap_or_default()));
        self
    }

    /// Configures database tracing.
    #[must_use]
    pub fn db(mut self, f: impl FnOnce(DbTracing) -> DbTracing) -> Self {
        self.db = Some(f(self.db.take().unwrap_or_default()));
        self
    }

    /// Configures mailer dispatch tracing.
    #[must_use]
    pub fn mailer(mut self, f: impl FnOnce(MailerTracing) -> MailerTracing) -> Self {
        self.mailer = Some(f(self.mailer.take().unwrap_or_default()));
        self
    }

    /// Configures migration / makemigrations tracing.
    #[must_use]
    pub fn migration(mut self, f: impl FnOnce(MigrationTracing) -> MigrationTracing) -> Self {
        self.migration = Some(f(self.migration.take().unwrap_or_default()));
        self
    }

    /// Configures template engine tracing.
    #[must_use]
    pub fn templates(mut self, f: impl FnOnce(TemplatesTracing) -> TemplatesTracing) -> Self {
        self.templates = Some(f(self.templates.take().unwrap_or_default()));
        self
    }

    /// Configures HTTP error-handling tracing.
    #[must_use]
    pub fn errors(mut self, f: impl FnOnce(ErrorsTracing) -> ErrorsTracing) -> Self {
        self.errors = Some(f(self.errors.take().unwrap_or_default()));
        self
    }

    /// Configures builder startup tracing.
    #[must_use]
    pub fn builder(mut self, f: impl FnOnce(BuilderTracing) -> BuilderTracing) -> Self {
        self.builder = Some(f(self.builder.take().unwrap_or_default()));
        self
    }

    /// Enables all categories at `DEBUG` level.
    ///
    /// No effect if `DEBUG` is not `true` or `1` in the environment —
    /// can be used unconditionally in `.with_log()`.
    ///
    /// ```rust,ignore
    /// .with_log(|l| l.dev())
    /// // or with override
    /// .with_log(|l| l.dev().db(|d| d.connect(Level::INFO)))
    /// ```
    #[must_use]
    pub fn dev(self) -> Self {
        if !crate::utils::env::is_debug() {
            return self;
        }
        self.forms(|f| f.dev())
            .middleware(|m| m.dev())
            .session(|s| s.dev())
            .auth(|a| a.dev())
            .admin(|a| a.dev())
            .db(|d| d.dev())
            .mailer(|m| m.dev())
            .migration(|m| m.dev())
            .templates(|t| t.dev())
            .errors(|e| e.dev())
            .builder(|b| b.dev())
    }
}

static LOG_CONFIG: Mutex<Option<Arc<RuniqueLog>>> = Mutex::new(None);

/// Initializes the log configuration — called once during `build()`.
/// Silent no-op if already initialized.
pub fn log_init(config: RuniqueLog) {
    if let Ok(mut guard) = LOG_CONFIG.lock()
        && guard.is_none()
    {
        *guard = Some(Arc::new(config));
    }
}

/// Returns the active log configuration.
/// Returns an empty config (all disabled) if `log_init` hasn't been called.
pub fn get_log() -> Arc<RuniqueLog> {
    LOG_CONFIG
        .lock()
        .ok()
        .and_then(|g| g.as_ref().cloned())
        .unwrap_or_default()
}

/// Resets the log configuration. Only call from tests.
pub fn reset_log_for_test() {
    if let Ok(mut guard) = LOG_CONFIG.lock() {
        *guard = None;
    }
}

/// Emits a tracing event at the configured dynamic level.
///
/// # Exemple
/// ```rust,ignore
/// if let Some(level) = get_log().middleware.as_ref().and_then(|m| m.csrf) {
///     runique_log!(level, path = %path, "csrf_token detected in a GET URL");
/// }
/// ```
#[macro_export]
macro_rules! runique_log {
    ($level:expr, $($args:tt)*) => {
        match $level {
            ::tracing::Level::ERROR => ::tracing::error!($($args)*),
            ::tracing::Level::WARN  => ::tracing::warn!($($args)*),
            ::tracing::Level::INFO  => ::tracing::info!($($args)*),
            ::tracing::Level::DEBUG => ::tracing::debug!($($args)*),
            _                       => ::tracing::trace!($($args)*),
        }
    };
}
