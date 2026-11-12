//! Centralized Runique log configuration — a tree of per-module tracing categories,
//! the `runique_log!` macro, and the global subscriber init.
//!
//! The configuration mirrors the framework module map: one sub-struct per subsystem
//! (`forms`, `middleware`, `session`, `auth`, `admin`, `db`, `mailer`, `migration`,
//! `templates`, `errors`, `builder`). Each leaf is an `Option<Level>` — `None` means
//! the event is disabled (zero cost).
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::{
    EnvFilter, Layer, Registry, fmt, fmt::format::FmtSpan, layer::SubscriberExt,
    util::SubscriberInitExt,
};

mod admin;
mod auth;
mod builder;
mod db;
mod errors;
mod forms;
mod mailer;
mod middleware;
mod migration;
mod output;
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
pub use output::{LogOutput, LogRecord, LogRotation, LogSink};
pub use session::SessionTracing;
pub use templates::TemplatesTracing;

/// Unified Runique log configuration — a tree of per-module categories.
///
/// # Exemple
/// ```rust,ignore
/// RuniqueApp::builder(config)
///     .with_log(|l| l
///         .subscriber_level("info")              // global default — RUST_LOG always wins
///         .middleware(|m| m.csrf(LogLevel::WARN).host_validation(LogLevel::INFO))
///         .session(|s| s.store(LogLevel::INFO))
///         .forms(|f| f.validate(LogLevel::DEBUG))
///         .admin(|a| a.crud(LogLevel::INFO))
///     )
/// ```
#[derive(Debug, Clone, Default)]
pub struct RuniqueLog {
    /// Tracing subscriber level. `RUST_LOG` takes priority if defined.
    /// Default: `"debug"` if `RUNIQUE_ENV=dev` or `DEBUG=true`, else `"warn"`.
    subscriber_level: Option<String>,

    /// Form pipeline tracing (`#[form]` fields, validation, render, finalize).
    pub forms: Option<FormTracing>,
    /// Middleware/security tracing (csrf, csp, cors, rate_limit, host_validation, …).
    pub middleware: Option<MiddlewareTracing>,
    /// Session lifecycle tracing (store, cleanup, exclusive_login).
    pub session: Option<SessionTracing>,
    /// Auth tracing (login, reset, password_init).
    pub auth: Option<AuthTracing>,
    /// Admin panel tracing (auth, crud, list, bulk).
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

    /// Log destinations. Empty means a single colored `Stdout` (the default).
    /// Add with [`output`](RuniqueLog::output) to fan out to console + file(s).
    outputs: Vec<LogOutput>,

    /// When `true`, the application owns the subscriber: Runique installs nothing
    /// (see [`external`](RuniqueLog::external)). Runique still emits its events.
    external: bool,

    /// When `true`, only the channels set by hand are on: the levels `.env`
    /// would give are not applied (see [`manual`](RuniqueLog::manual)).
    manual: bool,
}

impl RuniqueLog {
    /// Creates an empty log configuration: every category disabled, single colored stdout output.
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

    /// Adds a log destination. Repeatable — outputs are cumulative.
    ///
    /// With no `output`, logs go to a single colored `Stdout`. The
    /// `RUNIQUE_LOG_FILE` env var adds a file output at runtime without recompiling.
    ///
    /// ```rust,ignore
    /// .with_log(|l| l
    ///     .output(LogOutput::stdout())
    ///     .output(LogOutput::file("logs/app.json")))
    /// ```
    #[must_use]
    pub fn output(mut self, output: LogOutput) -> Self {
        self.outputs.push(output);
        self
    }

    /// Delegates the tracing subscriber to the application: Runique will **not**
    /// install one (no `try_init`), so you can build and install your own
    /// `tracing-subscriber` stack in `main`. Runique still emits its events to the
    /// `tracing` facade, so your subscriber receives them — filter the `runique`
    /// target out if you don't want them.
    ///
    /// Any [`output`](RuniqueLog::output) configured here is ignored in this mode
    /// (your subscriber decides where logs go).
    ///
    /// ```rust,ignore
    /// // main.rs — you own the subscriber
    /// tracing_subscriber::fmt()
    ///     .with_max_level(tracing::Level::INFO)
    ///     .init();
    ///
    /// RuniqueApp::builder(config)
    ///     .with_log(|l| l.external())
    ///     .build().await?
    ///     .run().await
    /// ```
    #[must_use]
    pub fn external(mut self) -> Self {
        self.external = true;
        self
    }

    /// Keeps only the channels set here: `DEBUG` and `RUNIQUE_ENV` no longer
    /// turn any channel on by themselves.
    ///
    /// ```rust,ignore
    /// .with_log(|l| l.manual().auth(|a| a.login(LogLevel::INFO)))
    /// ```
    #[must_use]
    pub fn manual(mut self) -> Self {
        self.manual = true;
        self
    }

    /// Turns on, for the channels left unset, what `.env` asks for. A channel
    /// set by hand keeps its level; [`manual`](RuniqueLog::manual) skips this.
    ///
    /// - `DEBUG=true`: every channel. The useful ones at `INFO`, the chatty
    ///   ones at `DEBUG` — each form field **with its value**, each admin
    ///   request and list. Passwords stay hidden; everything else a visitor
    ///   types is written to the logs, in production too.
    /// - `RUNIQUE_ENV=dev` without `DEBUG`: the useful ones only (sign-ins,
    ///   refusals, admin writes, HTTP errors, boot).
    /// - production without `DEBUG`: nothing more than what is set by hand.
    #[must_use]
    pub(crate) fn with_env_levels(mut self, dev: bool, debug: bool) -> Self {
        if self.manual || !(dev || debug) {
            return self;
        }
        use tracing::Level;
        fn fill(slot: &mut Option<Level>, level: Option<Level>) {
            if slot.is_none() {
                *slot = level;
            }
        }
        let useful = Some(Level::INFO);
        let chatty = debug.then_some(Level::DEBUG);

        let forms = self.forms.get_or_insert_with(Default::default);
        fill(&mut forms.field, chatty);
        fill(&mut forms.set_value, chatty);
        fill(&mut forms.validate, chatty);
        fill(&mut forms.render, chatty);
        fill(&mut forms.finalize, chatty);

        let middleware = self.middleware.get_or_insert_with(Default::default);
        fill(&mut middleware.csrf, useful);
        fill(&mut middleware.csp, useful);
        fill(&mut middleware.cors, useful);
        fill(&mut middleware.rate_limit, useful);
        fill(&mut middleware.host_validation, useful);
        fill(&mut middleware.open_redirect, useful);
        fill(&mut middleware.anti_bot, useful);
        fill(&mut middleware.https, useful);

        let session = self.session.get_or_insert_with(Default::default);
        fill(&mut session.store, useful);
        fill(&mut session.cleanup, useful);
        fill(&mut session.exclusive_login, useful);

        let auth = self.auth.get_or_insert_with(Default::default);
        fill(&mut auth.login, useful);
        fill(&mut auth.reset, useful);
        fill(&mut auth.password_init, useful);

        let admin = self.admin.get_or_insert_with(Default::default);
        fill(&mut admin.auth, chatty);
        fill(&mut admin.crud, useful);
        fill(&mut admin.list, chatty);
        fill(&mut admin.bulk, useful);

        let db = self.db.get_or_insert_with(Default::default);
        fill(&mut db.connect, useful);
        fill(&mut db.query, useful);

        let mailer = self.mailer.get_or_insert_with(Default::default);
        fill(&mut mailer.send, useful);

        let migration = self.migration.get_or_insert_with(Default::default);
        fill(&mut migration.plan, useful);
        fill(&mut migration.apply, useful);
        fill(&mut migration.rollback, useful);

        let templates = self.templates.get_or_insert_with(Default::default);
        fill(&mut templates.load, useful);
        fill(&mut templates.render, useful);

        let errors = self.errors.get_or_insert_with(Default::default);
        fill(&mut errors.http, useful);
        fill(&mut errors.render, useful);

        let builder = self.builder.get_or_insert_with(Default::default);
        fill(&mut builder.templates, useful);
        fill(&mut builder.registry, useful);
        fill(&mut builder.middleware, useful);
        fill(&mut builder.statics, useful);
        fill(&mut builder.routes, useful);

        self
    }

    /// Initializes the global tracing subscriber and returns the file-writer
    /// guards. Called automatically by `build()` — no effect if already initialized.
    ///
    /// The returned [`WorkerGuard`]s keep the non-blocking file-writer threads
    /// alive: they **must** be held for the lifetime of the app (stored in
    /// `RuniqueApp`), otherwise buffered log lines are dropped on shutdown.
    #[must_use]
    pub fn init_subscriber(&self) -> Vec<WorkerGuard> {
        // The application owns the subscriber: install nothing, keep emitting.
        if self.external && !self.outputs.is_empty() {
            tracing::warn!(
                "{}",
                crate::utils::trad::t("build.external_outputs_ignored")
            );
        }
        if self.external {
            return Vec::new();
        }

        // `DEBUG=true` has to let its channels through in production too.
        let default = self.subscriber_level.as_deref().unwrap_or_else(|| {
            if crate::utils::env::is_dev() || crate::utils::env::is_debug() {
                "debug"
            } else {
                "warn"
            }
        });

        let filter =
            std::env::var("RUST_LOG").map_or_else(|_| EnvFilter::new(default), EnvFilter::new);

        // Configured outputs, defaulting to a single stdout. The RUNIQUE_LOG_FILE
        // env var adds a file output at runtime (ops override, no recompile).
        let mut outputs = if self.outputs.is_empty() {
            vec![LogOutput::Stdout]
        } else {
            self.outputs.clone()
        };
        if let Ok(path) = std::env::var("RUNIQUE_LOG_FILE")
            && !path.is_empty()
        {
            outputs.push(LogOutput::file(path));
        }

        let (layers, guards) = Self::build_layers(outputs);

        let already_installed = tracing_subscriber::registry()
            .with(layers)
            .with(filter)
            .try_init()
            .is_err();

        // A global tracing subscriber can only be set once per process, and the library
        // forbids replacing it. If the host app already installed one, Runique's config
        // is inactive — surface it instead of failing silently (the warning is captured
        // by the already-installed subscriber).
        if already_installed {
            tracing::warn!(
                "{}",
                crate::utils::trad::t("build.subscriber_already_installed")
            );
            return Vec::new();
        }
        guards
    }

    /// Builds one `fmt` layer per output plus the matching non-blocking file
    /// guards. Split out from [`init_subscriber`](Self::init_subscriber) so it can
    /// be exercised with a thread-local subscriber in tests (the global one can
    /// only be installed once per process).
    fn build_layers(
        outputs: Vec<LogOutput>,
    ) -> (
        Vec<Box<dyn Layer<Registry> + Send + Sync>>,
        Vec<WorkerGuard>,
    ) {
        let mut guards: Vec<WorkerGuard> = Vec::new();
        let mut layers: Vec<Box<dyn Layer<Registry> + Send + Sync>> = Vec::new();

        for output in outputs {
            match output {
                LogOutput::Stdout => {
                    layers.push(fmt::layer().with_span_events(FmtSpan::CLOSE).boxed());
                }
                LogOutput::File { path, rotation } => {
                    let dir = path
                        .parent()
                        .filter(|p| !p.as_os_str().is_empty())
                        .map_or_else(|| PathBuf::from("."), Path::to_path_buf);
                    let Some(prefix) = path.file_name().map(std::ffi::OsStr::to_owned) else {
                        continue;
                    };
                    let appender = match rotation {
                        LogRotation::Daily => tracing_appender::rolling::daily(&dir, &prefix),
                        LogRotation::Hourly => tracing_appender::rolling::hourly(&dir, &prefix),
                        LogRotation::Never => tracing_appender::rolling::never(&dir, &prefix),
                    };
                    let (writer, guard) = tracing_appender::non_blocking(appender);
                    guards.push(guard);

                    let layer = fmt::layer()
                        .with_ansi(false)
                        .with_writer(writer)
                        .with_span_events(FmtSpan::CLOSE);
                    if LogOutput::is_json(&path) {
                        layers.push(layer.json().boxed());
                    } else {
                        layers.push(layer.boxed());
                    }
                }
                LogOutput::Custom(sink) => {
                    layers.push(output::SinkLayer::new(sink).boxed());
                }
            }
        }
        (layers, guards)
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
    /// No effect unless `RUNIQUE_ENV=dev` —
    /// can be used unconditionally in `.with_log()`.
    ///
    /// ```rust,ignore
    /// .with_log(|l| l.dev())
    /// // or with override
    /// .with_log(|l| l.dev().db(|d| d.connect(LogLevel::INFO)))
    /// ```
    #[must_use]
    pub fn dev(self) -> Self {
        if !crate::utils::env::is_dev() {
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

/// The level a log channel is set to, `None` when it is off:
/// `log_level!(session.store)` reads `get_log().session`, then its `store`.
macro_rules! log_level {
    ($domain:ident . $channel:ident) => {
        $crate::utils::runique_log::get_log()
            .$domain
            .as_ref()
            .and_then(|domain| domain.$channel)
    };
}
pub(crate) use log_level;

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

#[cfg(test)]
mod tests {
    use super::*;
    use tracing_subscriber::layer::SubscriberExt;

    /// Unique temp directory per test, so parallel runs don't clobber each other.
    fn unique_dir(tag: &str) -> PathBuf {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        let dir = std::env::temp_dir().join(format!("runique_log_test_{tag}_{nanos}"));
        std::fs::create_dir_all(&dir).expect("create temp dir");
        dir
    }

    #[test]
    fn output_is_repeatable_and_cumulative() {
        let log = RuniqueLog::new()
            .output(LogOutput::stdout())
            .output(LogOutput::file("logs/app.json"));
        assert_eq!(log.outputs.len(), 2);
    }

    #[test]
    fn external_installs_no_subscriber() {
        // external() must not call try_init (the app owns the subscriber): no guards.
        let guards = RuniqueLog::new().external().init_subscriber();
        assert!(guards.is_empty());
    }

    #[test]
    fn file_defaults_to_daily_rotation() {
        match LogOutput::file("logs/app.log") {
            LogOutput::File { path, rotation } => {
                assert_eq!(path, PathBuf::from("logs/app.log"));
                assert_eq!(rotation, LogRotation::Daily);
            }
            other => panic!("expected a File output, got {other:?}"),
        }
    }

    #[test]
    fn rotation_overrides_file_and_noops_on_stdout() {
        match LogOutput::file("a.log").rotation(LogRotation::Never) {
            LogOutput::File { rotation, .. } => assert_eq!(rotation, LogRotation::Never),
            other => panic!("expected a File output, got {other:?}"),
        }
        assert!(matches!(
            LogOutput::stdout().rotation(LogRotation::Hourly),
            LogOutput::Stdout
        ));
    }

    #[test]
    fn is_json_follows_the_extension() {
        assert!(LogOutput::is_json(Path::new("logs/app.json")));
        assert!(LogOutput::is_json(Path::new("APP.JSON"))); // case-insensitive
        assert!(!LogOutput::is_json(Path::new("app.log")));
        assert!(!LogOutput::is_json(Path::new("app")));
    }

    #[test]
    fn plain_file_receives_events_and_fields() {
        let dir = unique_dir("plain");
        let path = dir.join("app.log");
        // Never rotation → the file name is exactly `path` (no date suffix to resolve).
        let output = LogOutput::file(&path).rotation(LogRotation::Never);
        let (layers, guards) = RuniqueLog::build_layers(vec![output]);
        let subscriber = tracing_subscriber::registry().with(layers);
        tracing::subscriber::with_default(subscriber, || {
            tracing::error!(user = 42, "boom in plain file");
        });
        drop(guards); // flushes the non-blocking writer thread before we read

        let content = std::fs::read_to_string(&path).expect("log file written");
        assert!(content.contains("boom in plain file"), "got: {content}");
        assert!(content.contains("user"), "field missing: {content}");
        assert!(!content.contains('\u{1b}'), "ANSI escape leaked into file");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn json_file_is_structured() {
        let dir = unique_dir("json");
        let path = dir.join("app.json");
        let output = LogOutput::file(&path).rotation(LogRotation::Never);
        let (layers, guards) = RuniqueLog::build_layers(vec![output]);
        let subscriber = tracing_subscriber::registry().with(layers);
        tracing::subscriber::with_default(subscriber, || {
            tracing::warn!(code = 7, "structured line");
        });
        drop(guards);

        let content = std::fs::read_to_string(&path).expect("log file written");
        let first = content.lines().next().expect("at least one line");
        let v: serde_json::Value = serde_json::from_str(first).expect("each line is valid JSON");
        assert_eq!(v["level"], "WARN");
        assert_eq!(v["fields"]["message"], "structured line");
        assert_eq!(v["fields"]["code"], 7);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn custom_sink_receives_records() {
        type CapturedRecord = (tracing::Level, String, Vec<(&'static str, String)>);

        #[derive(Clone, Default)]
        struct Capture(Arc<std::sync::Mutex<Vec<CapturedRecord>>>);
        impl LogSink for Capture {
            fn log(&self, record: &LogRecord<'_>) {
                self.0.lock().unwrap().push((
                    record.level,
                    record.message.clone(),
                    record.fields.clone(),
                ));
            }
        }

        let cap = Capture::default();
        let (layers, guards) = RuniqueLog::build_layers(vec![LogOutput::sink(cap.clone())]);
        let subscriber = tracing_subscriber::registry().with(layers);
        tracing::subscriber::with_default(subscriber, || {
            tracing::info!(answer = 42, "hello sink");
        });
        drop(guards);

        let recs = cap.0.lock().unwrap();
        assert_eq!(recs.len(), 1);
        let (level, message, fields) = &recs[0];
        assert_eq!(*level, tracing::Level::INFO);
        assert_eq!(message, "hello sink");
        // `message` is split out; only `answer` remains in fields.
        assert_eq!(fields.as_slice(), &[("answer", "42".to_string())]);
    }

    /// What `.env` turns on for the channels left unset: nothing in
    /// production, the useful ones in development, all of them with `DEBUG`.
    #[test]
    fn env_levels_follow_debug_and_runique_env() {
        use tracing::Level;
        let (info, debug) = (Some(Level::INFO), Some(Level::DEBUG));
        // (useful channel, chatty channel) of a configuration.
        let seen = |log: &RuniqueLog| {
            (
                log.auth.as_ref().and_then(|a| a.login),
                log.forms.as_ref().and_then(|f| f.set_value),
                log.admin.as_ref().and_then(|a| a.list),
            )
        };

        let prod = RuniqueLog::new().with_env_levels(false, false);
        assert_eq!(seen(&prod), (None, None, None));

        let dev = RuniqueLog::new().with_env_levels(true, false);
        assert_eq!(seen(&dev), (info, None, None));

        for dev_too in [false, true] {
            let all = RuniqueLog::new().with_env_levels(dev_too, true);
            assert_eq!(seen(&all), (info, debug, debug), "dev = {dev_too}");
        }
    }

    #[test]
    fn a_level_set_by_hand_survives_the_env_levels() {
        use tracing::Level;
        let log = RuniqueLog::new()
            .auth(|a| a.login(Level::ERROR))
            .forms(|f| f.set_value(Level::TRACE))
            .with_env_levels(true, true);
        assert_eq!(log.auth.as_ref().and_then(|a| a.login), Some(Level::ERROR));
        assert_eq!(
            log.forms.as_ref().and_then(|f| f.set_value),
            Some(Level::TRACE)
        );
        // The channels beside them, left unset, still follow `.env`.
        assert_eq!(log.auth.as_ref().and_then(|a| a.reset), Some(Level::INFO));
        assert_eq!(log.forms.as_ref().and_then(|f| f.field), Some(Level::DEBUG));
    }

    #[test]
    fn manual_keeps_only_what_was_set_by_hand() {
        use tracing::Level;
        let log = RuniqueLog::new()
            .manual()
            .auth(|a| a.login(Level::WARN))
            .with_env_levels(true, true);
        assert_eq!(log.auth.as_ref().and_then(|a| a.login), Some(Level::WARN));
        assert_eq!(log.auth.as_ref().and_then(|a| a.reset), None);
        assert!(log.forms.is_none() && log.session.is_none());
    }

    // Written from cargo-mutants survivors (2026-10-08): each `dev()` turns
    // every channel of its category on, through each setter.
    #[test]
    fn dev_turns_every_channel_on() {
        use tracing::Level;
        let d = Some(Level::DEBUG);
        let a = AdminTracing::new().dev();
        assert_eq!([a.auth, a.crud, a.list, a.bulk], [d; 4]);
        let m = MiddlewareTracing::new().dev();
        assert_eq!(
            [
                m.csrf,
                m.csp,
                m.cors,
                m.rate_limit,
                m.host_validation,
                m.open_redirect,
                m.anti_bot,
                m.https
            ],
            [d; 8]
        );
        let db = DbTracing::new().dev();
        assert_eq!([db.connect, db.query], [d; 2]);
        let e = ErrorsTracing::new().dev();
        assert_eq!([e.http, e.render], [d; 2]);
        assert_eq!(MailerTracing::new().dev().send, d);
        let s = SessionTracing::new().dev();
        assert_eq!([s.store, s.cleanup, s.exclusive_login], [d; 3]);
    }

    #[test]
    fn migration_category_keeps_its_levels() {
        use tracing::Level;
        let log = RuniqueLog::new().migration(|m| m.plan(Level::INFO).rollback(Level::WARN));
        let m = log.migration.expect("configured");
        assert_eq!(
            (m.plan, m.apply, m.rollback),
            (Some(Level::INFO), None, Some(Level::WARN))
        );
    }

    #[test]
    fn sink_records_text_and_boolean_fields() {
        type Captured = Vec<(&'static str, String)>;
        #[derive(Clone, Default)]
        struct Capture(Arc<std::sync::Mutex<Vec<Captured>>>);
        impl LogSink for Capture {
            fn log(&self, record: &LogRecord<'_>) {
                self.0.lock().unwrap().push(record.fields.clone());
            }
        }
        let cap = Capture::default();
        let (layers, guards) = RuniqueLog::build_layers(vec![LogOutput::sink(cap.clone())]);
        let subscriber = tracing_subscriber::registry().with(layers);
        tracing::subscriber::with_default(subscriber, || {
            tracing::info!(who = "alice", ok = true, "fields");
        });
        drop(guards);
        let recs = cap.0.lock().unwrap();
        assert_eq!(
            recs[0].as_slice(),
            &[("who", "alice".to_string()), ("ok", "true".to_string())]
        );
    }

    #[test]
    fn log_outputs_describe_themselves() {
        assert_eq!(format!("{:?}", LogOutput::Stdout), "Stdout");
        let file = format!("{:?}", LogOutput::file("logs/app.log"));
        assert!(file.contains("logs/app.log"), "{file}");
    }
}
