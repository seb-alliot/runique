//! Tracing subscriber initialization — used by the CLI; web apps use `RuniqueLog`.
use crate::utils::runique_log::RuniqueLog;
use tracing_appender::non_blocking::WorkerGuard;

/// Initializes the tracing subscriber with default parameters.
/// Used by the Runique CLI — web applications don't need to call it
/// (the builder does it automatically via `RuniqueLog::init_subscriber`).
///
/// Returns the file-writer guards: the caller must keep them alive (bind to a
/// variable that lives until the program exits) or buffered file logs are lost.
#[must_use]
pub fn init_logging() -> Vec<WorkerGuard> {
    RuniqueLog::default().init_subscriber()
}
