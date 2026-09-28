//! Types of the test builder: `RuniqueTest`, its setup error, and the outcome
//! of each run (verdict, captured SQL trace, rendering).
use super::transaction_test::TestTransaction;
use std::fmt;
use std::{
    sync::{Arc, Mutex, atomic::AtomicUsize},
    time::Duration,
};

/// Why a `RuniqueTest` could not start.
#[derive(Debug)]
pub enum SetupError<E> {
    EnvFile(String),
    Engine(E),
}

impl<E: fmt::Display> fmt::Display for SetupError<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EnvFile(e) => write!(f, "cannot load env file {e}"),
            Self::Engine(e) => write!(f, "{e}"),
        }
    }
}

// Fields stay `pub(super)`: never `pub`, or a dev could write through
// `connection` directly and bypass the rollback.
pub struct RuniqueTest<C: TestTransaction> {
    pub(super) connection: C::Connection,
    pub(super) trace: TraceSink,
    pub(super) filter: Option<String>,
    /// Every test name met, filtered out or not — lists the valid names when
    /// the requested one does not exist.
    pub(super) seen: Mutex<Vec<String>>,
    pub(super) passed: AtomicUsize,
    pub(super) failed: AtomicUsize,
    /// Serializes runs: one transaction at a time (SQLite pools hold a single
    /// connection, and the trace buffer is shared).
    pub(super) serial: tokio::sync::Mutex<()>,
}

/// One statement executed during a test, captured by the connection's metric callback.
pub struct QueryTrace {
    /// SQL with its bound values inlined, password hashes already masked.
    pub sql: String,
    pub elapsed: Duration,
    pub failed: bool,
}

/// Buffer the connection pushes every executed statement into.
pub type TraceSink = Arc<Mutex<Vec<QueryTrace>>>;

/// Verdict of a test: the handler returned `Ok`, or the error it returned.
pub enum Reason<E> {
    Win,
    Error(E),
}

/// Everything a test produced, printed through its `Display` impl.
pub struct FormatResult<E> {
    pub name_test: String,
    pub reason: Reason<E>,
    pub trace: Vec<QueryTrace>,
    /// Why the transaction could not be rolled back explicitly, if it couldn't.
    pub rollback_error: Option<String>,
}

impl<E> FormatResult<E> {
    /// A test passes only if its handler succeeded and its transaction was rolled back.
    pub fn passed(&self) -> bool {
        matches!(self.reason, Reason::Win) && self.rollback_error.is_none()
    }
}

impl<E: fmt::Display> fmt::Display for FormatResult<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.reason {
            Reason::Win => writeln!(f, "  ✓ {}", self.name_test)?,
            Reason::Error(e) => writeln!(f, "  ✗ {} — {e}", self.name_test)?,
        }
        for (n, query) in (1..).zip(&self.trace) {
            let failed = if query.failed { "  ✗" } else { "" };
            writeln!(
                f,
                "      {n}. {} ({:.1} ms){failed}",
                query.sql,
                query.elapsed.as_secs_f64() * 1000.0
            )?;
        }
        if let Some(e) = &self.rollback_error {
            writeln!(f, "      rollback: {e}")?;
        }
        Ok(())
    }
}
