//! Centralized Runique log configuration — levels by category, `runique_log!` macro, `dev()` helper.
use std::sync::OnceLock;
use tracing::Level;
use tracing_subscriber::{EnvFilter, fmt::format::FmtSpan};

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
    }
}

static LOG_CONFIG: OnceLock<RuniqueLog> = OnceLock::new();

/// Initializes the log configuration — called once during `build()`.
pub fn log_init(config: RuniqueLog) {
    // Double silent call: initial config is kept.
    LOG_CONFIG.set(config).ok();
}

/// Returns the active log configuration.
/// Returns an empty config (all disabled) if `log_init` hasn't been called.
pub fn get_log() -> &'static RuniqueLog {
    LOG_CONFIG.get_or_init(RuniqueLog::default)
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
