//! Tracing subscriber initialization — used by the CLI; web apps use `RuniqueLog`.
use crate::utils::runique_log::RuniqueLog;

/// Initializes the tracing subscriber with default parameters.
/// Used by the Runique CLI — web applications don't need to call it
/// (the builder does it automatically via `RuniqueLog::init_subscriber`).
pub fn init_logging() {
    RuniqueLog::default().init_subscriber();
}
