//! Log output destinations for the Runique subscriber.
//!
//! A [`RuniqueLog`](super::RuniqueLog) can fan out to several outputs at once
//! (e.g. colored console on the VPS + a JSON file for ingestion + a custom sink):
//!
//! ```rust,ignore
//! .with_log(|l| l
//!     .dev()
//!     .output(LogOutput::stdout())              // console, ANSI colors
//!     .output(LogOutput::file("logs/app.json")) // .json → JSON, one event per line
//!     .output(LogOutput::file("logs/app.log").rotation(LogRotation::Daily))
//!     .output(LogOutput::sink(MyDatabaseSink::new()))) // dev-provided destination
//! ```
//!
//! The on-disk **format is inferred from the file extension**: `.json` produces
//! structured JSON, anything else produces the same plain layout as the console
//! (without ANSI escapes). File writing is non-blocking — see the `WorkerGuard`
//! returned by [`RuniqueLog::init_subscriber`](super::RuniqueLog::init_subscriber).
//!
//! Runique ships **no database sink** on purpose (writing every event to the main
//! DB overloads it and is awkward to query). Instead, [`LogSink`] is the escape
//! hatch: a dev who wants a DB/HTTP/queue sink implements it without ever touching
//! `tracing` types. Async destinations buffer into their own channel inside `log`.
use std::path::PathBuf;
use std::sync::Arc;
use tracing::field::{Field, Visit};
use tracing::{Event, Level, Subscriber};
use tracing_subscriber::Layer;
use tracing_subscriber::layer::Context;

/// How a file output rolls over to a new file over time.
///
/// `tracing-appender` appends the period to the file name
/// (`app.log.2026-06-16` for [`Daily`](LogRotation::Daily)).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum LogRotation {
    /// One file per day.
    #[default]
    Daily,
    /// One file per hour.
    Hourly,
    /// A single file, never rolled.
    Never,
}

/// A single log destination.
///
/// Build with [`LogOutput::stdout`] / [`LogOutput::file`] / [`LogOutput::sink`]
/// and add as many as needed via [`RuniqueLog::output`](super::RuniqueLog::output).
#[derive(Clone)]
pub enum LogOutput {
    /// The process standard output, with ANSI colors.
    Stdout,
    /// A rolling file. The format (JSON vs plain) is inferred from `path`'s extension.
    File {
        /// Full path, e.g. `logs/app.json`. The parent directory is the rolling
        /// directory and the file name is the rolling prefix.
        path: PathBuf,
        /// Roll-over policy. Defaults to [`LogRotation::Daily`].
        rotation: LogRotation,
    },
    /// A custom, dev-provided destination (see [`LogSink`]).
    Custom(Arc<dyn LogSink>),
}

// `dyn LogSink` is not `Debug`, so `LogOutput` can't derive it.
impl std::fmt::Debug for LogOutput {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Stdout => f.write_str("Stdout"),
            Self::File { path, rotation } => f
                .debug_struct("File")
                .field("path", path)
                .field("rotation", rotation)
                .finish(),
            Self::Custom(_) => f.write_str("Custom(<sink>)"),
        }
    }
}

impl LogOutput {
    /// Console output with colors.
    #[must_use]
    pub fn stdout() -> Self {
        Self::Stdout
    }

    /// File output (non-blocking, daily rotation by default).
    /// JSON if the path ends in `.json`, otherwise plain text.
    #[must_use]
    pub fn file(path: impl Into<PathBuf>) -> Self {
        Self::File {
            path: path.into(),
            rotation: LogRotation::Daily,
        }
    }

    /// A custom destination implementing [`LogSink`] (e.g. a database or HTTP sink).
    #[must_use]
    pub fn sink(sink: impl LogSink) -> Self {
        Self::Custom(Arc::new(sink))
    }

    /// Overrides the rotation policy of a file output (no-op on other variants).
    #[must_use]
    pub fn rotation(mut self, rotation: LogRotation) -> Self {
        if let Self::File { rotation: r, .. } = &mut self {
            *r = rotation;
        }
        self
    }

    /// True when the file path requests JSON output (`.json` extension).
    pub(super) fn is_json(path: &std::path::Path) -> bool {
        path.extension()
            .and_then(|e| e.to_str())
            .is_some_and(|e| e.eq_ignore_ascii_case("json"))
    }
}

// ─── Custom sink ──────────────────────────────────────────────────────────────

/// A single log event, flattened for a [`LogSink`] — no `tracing` types exposed.
#[derive(Debug)]
pub struct LogRecord<'a> {
    /// Severity of the event.
    pub level: Level,
    /// Event target (usually the emitting module path).
    pub target: &'a str,
    /// Source file the event was emitted from, if available.
    pub file: Option<&'a str>,
    /// Source line, if available.
    pub line: Option<u32>,
    /// The human-readable message (the `message` field).
    pub message: String,
    /// Remaining structured fields, rendered to strings (`message` excluded).
    pub fields: Vec<(&'static str, String)>,
}

/// A dev-provided log destination.
///
/// Implement this to send Runique (and your app's) log events anywhere — a
/// database table, an HTTP collector, a message queue — and register it with
/// [`LogOutput::sink`]. [`log`](LogSink::log) is called **synchronously** on the
/// emitting thread, so an async sink must hand the record off to its own channel
/// and drain it from a background task (it owns that task's lifecycle).
///
/// ```rust,ignore
/// struct DbSink { tx: tokio::sync::mpsc::Sender<OwnedLog> }
/// impl LogSink for DbSink {
///     fn log(&self, record: &LogRecord) {
///         // never block here — just enqueue
///         let _ = self.tx.try_send(OwnedLog::from(record));
///     }
/// }
/// ```
pub trait LogSink: Send + Sync + 'static {
    /// Receives one log event. Must not block.
    fn log(&self, record: &LogRecord<'_>);
}

/// Internal `tracing` layer adapting an [`Event`] into a [`LogRecord`] for a [`LogSink`].
pub(super) struct SinkLayer(Arc<dyn LogSink>);

impl SinkLayer {
    pub(super) fn new(sink: Arc<dyn LogSink>) -> Self {
        Self(sink)
    }
}

impl<S: Subscriber> Layer<S> for SinkLayer {
    fn on_event(&self, event: &Event<'_>, _ctx: Context<'_, S>) {
        let meta = event.metadata();
        let mut visitor = RecordVisitor::default();
        event.record(&mut visitor);
        let record = LogRecord {
            level: *meta.level(),
            target: meta.target(),
            file: meta.file(),
            line: meta.line(),
            message: visitor.message,
            fields: visitor.fields,
        };
        self.0.log(&record);
    }
}

/// Collects an event's fields into owned strings, keeping `message` apart.
#[derive(Default)]
struct RecordVisitor {
    message: String,
    fields: Vec<(&'static str, String)>,
}

impl Visit for RecordVisitor {
    fn record_debug(&mut self, field: &Field, value: &dyn std::fmt::Debug) {
        if field.name() == "message" {
            self.message = format!("{value:?}");
        } else {
            self.fields.push((field.name(), format!("{value:?}")));
        }
    }

    fn record_str(&mut self, field: &Field, value: &str) {
        self.fields.push((field.name(), value.to_string()));
    }

    fn record_i64(&mut self, field: &Field, value: i64) {
        self.fields.push((field.name(), value.to_string()));
    }

    fn record_u64(&mut self, field: &Field, value: u64) {
        self.fields.push((field.name(), value.to_string()));
    }

    fn record_bool(&mut self, field: &Field, value: bool) {
        self.fields.push((field.name(), value.to_string()));
    }

    fn record_f64(&mut self, field: &Field, value: f64) {
        self.fields.push((field.name(), value.to_string()));
    }
}
