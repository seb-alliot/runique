//! Centralized Runique log configuration — levels by category, `runique_log!` macro, `dev()` helper.
use std::sync::{Arc, Mutex};
use tracing::Level;
use tracing_subscriber::{EnvFilter, fmt::format::FmtSpan};

/// Form pipeline tracing — each field covers one stage of `#[form]` processing.
#[derive(Debug, Clone, Default)]
pub struct FormTracing {
    /// Field-level events: type resolution, coercion, missing/extra fields.
    pub field: Option<Level>,
    /// `set_value()` calls: raw input → typed value assignment.
    pub set_value: Option<Level>,
    /// `validate()` results: per-field errors, required/length/format checks.
    pub validate: Option<Level>,
    /// HTML render events: widget selection, context injection.
    pub render: Option<Level>,
    /// `finalize()` per field: password hashing, file move to MEDIA_ROOT.
    pub finalize: Option<Level>,
}

impl FormTracing {
    pub fn new() -> Self {
        Self::default()
    }
    #[must_use]
    pub fn field(mut self, level: Level) -> Self {
        self.field = Some(level);
        self
    }
    #[must_use]
    pub fn set_value(mut self, level: Level) -> Self {
        self.set_value = Some(level);
        self
    }
    #[must_use]
    pub fn validate(mut self, level: Level) -> Self {
        self.validate = Some(level);
        self
    }
    #[must_use]
    pub fn render(mut self, level: Level) -> Self {
        self.render = Some(level);
        self
    }
    #[must_use]
    pub fn finalize(mut self, level: Level) -> Self {
        self.finalize = Some(level);
        self
    }
    pub fn dev(self) -> Self {
        self.field(Level::DEBUG)
            .set_value(Level::DEBUG)
            .validate(Level::DEBUG)
            .render(Level::DEBUG)
            .finalize(Level::DEBUG)
    }
}

/// Builder startup tracing — one-time events during `build()`.
#[derive(Debug, Clone, Default)]
pub struct BuilderTracing {
    /// Template loading: nb internal + user templates registered in Tera.
    pub templates: Option<Level>,
    /// Admin registry: nb resources registered at startup.
    pub registry: Option<Level>,
    /// Middleware stack: each slot name + number assigned at startup.
    pub middleware: Option<Level>,
    /// Static files: static_url + path + media_url + path.
    pub statics: Option<Level>,
    /// URL routes: nb named routes in registry after `add_urls()`.
    pub routes: Option<Level>,
}

impl BuilderTracing {
    pub fn new() -> Self {
        Self::default()
    }
    #[must_use]
    pub fn templates(mut self, level: Level) -> Self {
        self.templates = Some(level);
        self
    }
    #[must_use]
    pub fn registry(mut self, level: Level) -> Self {
        self.registry = Some(level);
        self
    }
    #[must_use]
    pub fn middleware(mut self, level: Level) -> Self {
        self.middleware = Some(level);
        self
    }
    #[must_use]
    pub fn statics(mut self, level: Level) -> Self {
        self.statics = Some(level);
        self
    }
    #[must_use]
    pub fn routes(mut self, level: Level) -> Self {
        self.routes = Some(level);
        self
    }
    pub fn dev(self) -> Self {
        self.templates(Level::DEBUG)
            .registry(Level::DEBUG)
            .middleware(Level::DEBUG)
            .statics(Level::DEBUG)
            .routes(Level::DEBUG)
    }
}

/// Auth tracing — session lifecycle events.
#[derive(Debug, Clone, Default)]
pub struct AuthTracing {
    /// User login: session creation, group loading, DB persistence, exclusive flag.
    pub login: Option<Level>,
    /// Password reset flow: token generated, email sent, token validated/consumed, password updated.
    pub reset: Option<Level>,
}

impl AuthTracing {
    pub fn new() -> Self {
        Self::default()
    }
    #[must_use]
    pub fn login(mut self, level: Level) -> Self {
        self.login = Some(level);
        self
    }
    #[must_use]
    pub fn reset(mut self, level: Level) -> Self {
        self.reset = Some(level);
        self
    }
    pub fn dev(self) -> Self {
        self.login(Level::DEBUG).reset(Level::DEBUG)
    }
}

/// Mailer tracing — email dispatch events.
#[derive(Debug, Clone, Default)]
pub struct MailerTracing {
    /// `Email::send()`: backend used, recipient, subject, result (ok/err).
    pub send: Option<Level>,
}

impl MailerTracing {
    pub fn new() -> Self {
        Self::default()
    }
    #[must_use]
    pub fn send(mut self, level: Level) -> Self {
        self.send = Some(level);
        self
    }
    pub fn dev(self) -> Self {
        self.send(Level::DEBUG)
    }
}

/// Admin panel tracing — per-operation granularity.
#[derive(Debug, Clone, Default)]
pub struct AdminTracing {
    /// Auth checks: login, permission gate, write-access guard.
    pub auth: Option<Level>,
    /// CRUD handlers: detail, create, edit, delete — request + outcome.
    pub crud: Option<Level>,
    /// List view: pagination, ordering, column resolution.
    pub list: Option<Level>,
    /// Bulk operations: group_action, group_set, bulk_delete.
    pub bulk: Option<Level>,
}

impl AdminTracing {
    pub fn new() -> Self {
        Self::default()
    }
    #[must_use]
    pub fn auth(mut self, level: Level) -> Self {
        self.auth = Some(level);
        self
    }
    #[must_use]
    pub fn crud(mut self, level: Level) -> Self {
        self.crud = Some(level);
        self
    }
    #[must_use]
    pub fn list(mut self, level: Level) -> Self {
        self.list = Some(level);
        self
    }
    #[must_use]
    pub fn bulk(mut self, level: Level) -> Self {
        self.bulk = Some(level);
        self
    }
    pub fn dev(self) -> Self {
        self.auth(Level::DEBUG)
            .crud(Level::DEBUG)
            .list(Level::DEBUG)
            .bulk(Level::DEBUG)
    }
}

/// Unified Runique log configuration.
///
/// Controls both the global tracing subscriber level and internal framework
/// categories.
///
/// # Exemple
/// ```rust,ignore
/// RuniqueApp::builder(config)
///     .with_log(|l| l
///         .subscriber_level("info")   // optional — default: debug/warn according to DEBUG env
///         .csrf(Level::WARN)
///         .session(Level::INFO)
///         .forms(|f| f.validate(Level::DEBUG))
///         .admin(|a| a.crud(Level::INFO))
///     )
/// ```
#[derive(Debug, Clone, Default)]
pub struct RuniqueLog {
    /// Tracing subscriber level. `RUST_LOG` takes priority if defined.
    /// Default: `"debug"` if `DEBUG=true`, else `"warn"`.
    subscriber_level: Option<String>,

    /// Detects a `csrf_token` in a GET URL (silent cleanup).
    pub csrf: Option<Level>,
    /// Traces session invalidation during exclusive login.
    pub exclusive_login: Option<Level>,
    /// Reports `filter_fn` failure in admin list view.
    pub filter_fn: Option<Level>,
    /// Reports admin roles registry access errors.
    pub roles: Option<Level>,
    /// Warns if `password_init()` is called multiple times.
    pub password_init: Option<Level>,
    /// Session store traces: memory watermarks, large records, cleanup errors.
    pub session: Option<Level>,
    /// DB connection info (connecting / connected successfully).
    pub db: Option<Level>,
    /// Host header validation rejections (HTTP/2 `:authority` fallback included).
    pub host_validation: Option<Level>,
    /// ACME/TLS lifecycle events: cert loaded, renewed, binding port 443.
    pub acme: Option<Level>,

    /// Form pipeline tracing (`#[form]` fields, validation, render, finalize).
    pub forms: Option<FormTracing>,
    /// Admin panel tracing (auth, CRUD, list, bulk).
    pub admin: Option<AdminTracing>,
    /// Auth tracing (login session lifecycle).
    pub auth: Option<AuthTracing>,
    /// Mailer tracing (email dispatch results).
    pub mailer: Option<MailerTracing>,
    /// Builder startup tracing (templates, registry, middleware slots).
    pub builder: Option<BuilderTracing>,
    /// Rate limiter: requests blocked (ip, retry_after).
    pub rate_limit: Option<Level>,
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

        tracing_subscriber::fmt()
            .with_env_filter(filter)
            .with_span_events(FmtSpan::CLOSE)
            .try_init()
            .ok();
    }
    #[must_use]
    pub fn csrf(mut self, level: Level) -> Self {
        self.csrf = Some(level);
        self
    }
    #[must_use]
    pub fn exclusive_login(mut self, level: Level) -> Self {
        self.exclusive_login = Some(level);
        self
    }
    #[must_use]
    pub fn filter_fn(mut self, level: Level) -> Self {
        self.filter_fn = Some(level);
        self
    }
    #[must_use]
    pub fn roles(mut self, level: Level) -> Self {
        self.roles = Some(level);
        self
    }
    #[must_use]
    pub fn password_init(mut self, level: Level) -> Self {
        self.password_init = Some(level);
        self
    }
    #[must_use]
    pub fn session(mut self, level: Level) -> Self {
        self.session = Some(level);
        self
    }
    #[must_use]
    pub fn db(mut self, level: Level) -> Self {
        self.db = Some(level);
        self
    }
    #[must_use]
    pub fn host_validation(mut self, level: Level) -> Self {
        self.host_validation = Some(level);
        self
    }
    #[must_use]
    pub fn acme(mut self, level: Level) -> Self {
        self.acme = Some(level);
        self
    }

    /// Configures form pipeline tracing.
    ///
    /// ```rust,ignore
    /// .with_log(|l| l.forms(|f| f.validate(Level::DEBUG).set_value(Level::TRACE)))
    /// ```
    #[must_use]
    pub fn forms(mut self, f: impl FnOnce(FormTracing) -> FormTracing) -> Self {
        self.forms = Some(f(self.forms.take().unwrap_or_default()));
        self
    }

    /// Configures admin panel tracing.
    ///
    /// ```rust,ignore
    /// .with_log(|l| l.admin(|a| a.crud(Level::INFO).auth(Level::WARN)))
    /// ```
    #[must_use]
    pub fn admin(mut self, f: impl FnOnce(AdminTracing) -> AdminTracing) -> Self {
        self.admin = Some(f(self.admin.take().unwrap_or_default()));
        self
    }

    /// Configures auth session tracing.
    ///
    /// ```rust,ignore
    /// .with_log(|l| l.auth(|a| a.login(Level::INFO)))
    /// ```
    #[must_use]
    pub fn auth(mut self, f: impl FnOnce(AuthTracing) -> AuthTracing) -> Self {
        self.auth = Some(f(self.auth.take().unwrap_or_default()));
        self
    }

    /// Configures mailer dispatch tracing.
    ///
    /// ```rust,ignore
    /// .with_log(|l| l.mailer(|m| m.send(Level::INFO)))
    /// ```
    #[must_use]
    pub fn mailer(mut self, f: impl FnOnce(MailerTracing) -> MailerTracing) -> Self {
        self.mailer = Some(f(self.mailer.take().unwrap_or_default()));
        self
    }

    /// Rate limiter: logs blocked requests (ip, retry_after_secs).
    #[must_use]
    pub fn rate_limit(mut self, level: Level) -> Self {
        self.rate_limit = Some(level);
        self
    }

    /// Configures builder startup tracing.
    ///
    /// ```rust,ignore
    /// .with_log(|l| l.builder(|b| b.templates(Level::DEBUG).middleware(Level::DEBUG)))
    /// ```
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
    /// .with_log(|l| l.dev().db(Level::INFO))
    /// ```
    #[must_use]
    pub fn dev(self) -> Self {
        if !crate::utils::env::is_debug() {
            return self;
        }
        self.csrf(Level::DEBUG)
            .exclusive_login(Level::DEBUG)
            .filter_fn(Level::DEBUG)
            .roles(Level::DEBUG)
            .password_init(Level::DEBUG)
            .session(Level::DEBUG)
            .db(Level::DEBUG)
            .host_validation(Level::DEBUG)
            .acme(Level::DEBUG)
            .rate_limit(Level::DEBUG)
            .forms(|f| f.dev())
            .admin(|a| a.dev())
            .auth(|a| a.dev())
            .mailer(|m| m.dev())
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
/// if let Some(level) = get_log().csrf {
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
