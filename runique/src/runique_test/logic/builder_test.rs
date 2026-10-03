//! `runique_test`: runs one piece of business logic against a real database,
//! inside a transaction that always gets rolled back, and prints the SQL it ran.
use super::struct_test::{
    FormatResult, Issue, QueryTrace, Reason, TestFailure, TraceSink, msg, msgf,
};
use super::transaction_test::TestTransaction;
use crate::utils::aliases::StrMap;
use futures_util::FutureExt;
use std::any::Any;
use std::panic::AssertUnwindSafe;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::Duration;

/// One test at a time across the whole test process: cargo runs tests on
/// several threads, and a SQLite pool only has one connection.
static SERIAL: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());

/// The last target announced, so it's shown again only when it changes.
static LAST_TARGET: Mutex<Option<String>> = Mutex::new(None);

/// The env file key that overrides [`DEFAULT_TIMEOUT`], in seconds.
pub const TIMEOUT_KEY: &str = "RUNIQUE_TEST_TIMEOUT";
const DEFAULT_TIMEOUT: Duration = Duration::from_secs(30);
/// How long the rollback and the transaction check get once the handler is done.
const CLEANUP_TIMEOUT: Duration = Duration::from_secs(15);

/// What the running test reports to outside the database: its SQL trace.
pub(crate) struct TestScope {
    trace: TraceSink,
}

tokio::task_local! {
    /// Set for the whole handler, so code deeper down can report to it
    /// (`ADb::execute_unprepared`, `expect_db_error`),
    /// and so a `runique_test` started from inside a handler can tell.
    static CURRENT: Arc<TestScope>;
}

/// Runs `handler` in a fresh transaction on the database `env_file` points to,
/// rolls back, then prints the result and the SQL trace.
///
/// Meant to be the last expression of a `#[tokio::test]` that returns
/// `Result<(), TestFailure>`. The handler only borrows the connection, so once
/// it's done this function owns it again and can roll it back.
///
/// Never panics: a handler that errors, panics or runs out of time still gets
/// its transaction rolled back, and every failure (setup included) comes back
/// as `Err`. On top of what the handler returns, the test fails when a query
/// failed but the handler still returned `Ok` (an error got swallowed; use
/// [`expect_db_error`](crate::runique_test::expect_db_error) for failures you
/// want), or when the transaction was ended from inside the test.
pub async fn runique_test<C: TestTransaction>(
    env_file: &str,
    handler: impl AsyncFnOnce(&C) -> Result<(), C::Error>,
) -> Result<(), TestFailure> {
    let name_test = current_test_name();

    // `SERIAL` isn't reentrant: a nested call would wait for its own caller forever.
    if CURRENT.try_with(|_| ()).is_ok() {
        return report(FormatResult::<C::Error> {
            name_test,
            reason: Reason::Setup(msg("runique_test.nested").into_owned()),
            trace: Vec::new(),
            issues: Vec::new(),
        });
    }

    let _serial = SERIAL.lock().await;
    let trace = TraceSink::default();
    let scope = Arc::new(TestScope {
        trace: trace.clone(),
    });

    let (reason, issues) = CURRENT
        .scope(scope.clone(), run::<C>(env_file, &trace, handler))
        .await;

    report(FormatResult {
        name_test,
        reason,
        trace: std::mem::take(&mut *lock(&trace)),
        issues,
    })
}

/// Prints the result and turns it into what the test returns.
fn report<E: std::fmt::Display>(result: FormatResult<E>) -> Result<(), TestFailure> {
    print!("{result}");
    if result.passed() {
        return Ok(());
    }
    Err(TestFailure {
        message: result.failure_message().unwrap_or_default(),
        name_test: result.name_test,
    })
}

async fn run<C: TestTransaction>(
    env_file: &str,
    trace: &TraceSink,
    handler: impl AsyncFnOnce(&C) -> Result<(), C::Error>,
) -> (Reason<C::Error>, Vec<Issue>) {
    let (_connection, db, timeout) = match open::<C>(env_file, trace).await {
        Ok(opened) => opened,
        Err(setup) => return (Reason::Setup(setup), Vec::new()),
    };

    // After a panic or a timeout, the only thing still used is the
    // transaction, and all that happens to it is a check and a rollback.
    let outcome =
        tokio::time::timeout(timeout, AssertUnwindSafe(handler(&db)).catch_unwind()).await;
    let reason = match outcome {
        Ok(Ok(Ok(()))) => Reason::Win,
        Ok(Ok(Err(e))) => Reason::Error(e),
        Ok(Err(payload)) => Reason::Panic(panic_message(payload.as_ref())),
        Err(_) => Reason::TimedOut(timeout.as_secs()),
    };

    let mut issues = Vec::new();
    if matches!(reason, Reason::Win) {
        let swallowed = lock(trace)
            .iter()
            .filter(|q| q.failed && !q.expected)
            .count();
        if swallowed > 0 {
            issues.push(Issue::SwallowedFailures(swallowed));
        }
    }

    // A timed-out handler may have left the connection mid-query: skip the
    // check, the rollback below will tell.
    if !matches!(reason, Reason::TimedOut(_)) {
        let before = lock(trace).len();
        if let Ok(Ok(false)) = tokio::time::timeout(CLEANUP_TIMEOUT, db.still_open()).await {
            issues.push(Issue::TransactionEnded);
        }
        // The check's own queries aren't part of what the test did.
        lock(trace).truncate(before);
    }

    match tokio::time::timeout(CLEANUP_TIMEOUT, db.rollback_test()).await {
        Ok(Ok(())) => {}
        Ok(Err(e)) => issues.push(Issue::RollbackFailed(e.to_string())),
        Err(_) => issues.push(Issue::RollbackFailed(msgf(
            "runique_test.rollback_timed_out",
            &[CLEANUP_TIMEOUT.as_secs()],
        ))),
    }
    (reason, issues)
}

/// Everything before the handler runs. The connection is handed back too,
/// so it outlives the transaction opened on it.
async fn open<C: TestTransaction>(
    env_file: &str,
    trace: &TraceSink,
) -> Result<(C::Connection, C, Duration), String> {
    let path = resolve(env_file);
    let vars = read_env_file(&path)?;
    let timeout = timeout_from(&vars)?;
    let config = C::load_config(&vars).map_err(|e| e.to_string())?;
    announce(&format!("{} ({})", C::describe(&config), path.display()));

    let connection = C::connect(&config, trace.clone())
        .await
        .map_err(|e| msgf("runique_test.connect_failed", &[e]))?;
    let db = C::begin_test(&connection)
        .await
        .map_err(|e| msgf("runique_test.begin_failed", &[e]))?;
    Ok((connection, db, timeout))
}

/// Shows the target database, but only when it differs from the last one:
/// two env files in the same run each get announced.
fn announce(target: &str) {
    let mut last = LAST_TARGET.lock().unwrap_or_else(|e| e.into_inner());
    if last.as_deref() != Some(target) {
        // Leading newline: cargo has already printed `test … ...` on this line.
        println!("\nrunique test → {target}");
        *last = Some(target.to_string());
    }
}

/// A relative path is read from the package being tested, not from wherever
/// the test runner happens to start: `cargo test` sets `CARGO_MANIFEST_DIR`,
/// other runners may not start in the package directory.
fn resolve(env_file: &str) -> PathBuf {
    let path = Path::new(env_file);
    match std::env::var_os("CARGO_MANIFEST_DIR") {
        Some(dir) if path.is_relative() => Path::new(&dir).join(path),
        _ => path.to_path_buf(),
    }
}

fn timeout_from(vars: &StrMap) -> Result<Duration, String> {
    match vars.get(TIMEOUT_KEY) {
        None => Ok(DEFAULT_TIMEOUT),
        Some(raw) => match raw.trim().parse::<u64>() {
            Ok(secs) if secs > 0 => Ok(Duration::from_secs(secs)),
            _ => Err(msgf(
                "runique_test.bad_timeout",
                &[TIMEOUT_KEY, raw.as_str()],
            )),
        },
    }
}

/// Adds a statement to the running test's trace, if there is one. For what
/// SeaORM's metric callback doesn't report (`execute_unprepared`).
pub(crate) fn record(sql: &str, elapsed: Duration, failed: bool) {
    let _ = CURRENT.try_with(|scope| {
        lock(&scope.trace).push(QueryTrace {
            sql: sql.to_string(),
            elapsed,
            failed,
            expected: false,
        });
    });
}

/// How many statements the running test's trace holds so far (0 outside a test).
pub(crate) fn trace_len() -> usize {
    CURRENT
        .try_with(|scope| lock(&scope.trace).len())
        .unwrap_or(0)
}

/// Marks every statement from `start` on as expected to fail.
pub(crate) fn mark_expected_from(start: usize) {
    let _ = CURRENT.try_with(|scope| {
        for query in lock(&scope.trace).iter_mut().skip(start) {
            query.expected = true;
        }
    });
}

fn lock(trace: &TraceSink) -> std::sync::MutexGuard<'_, Vec<QueryTrace>> {
    trace.lock().unwrap_or_else(|e| e.into_inner())
}

/// What `panic!` was called with: a `&str` or a `String` in practice.
fn panic_message(payload: &(dyn Any + Send)) -> String {
    payload
        .downcast_ref::<&str>()
        .map(|s| s.to_string())
        .or_else(|| payload.downcast_ref::<String>().cloned())
        .unwrap_or_else(|| msg("runique_test.no_panic_message").into_owned())
}

/// The running test's name, taken from the thread cargo's test harness starts
/// for it: `runique_test::user::add_email` shows up as `user::add_email`.
fn current_test_name() -> String {
    let name = std::thread::current()
        .name()
        .unwrap_or("unnamed test")
        .to_string();
    name.strip_prefix("runique_test::")
        .map(str::to_string)
        .unwrap_or(name)
}

/// Reads `path` into key/value pairs without touching the process
/// environment, which other test threads may be reading at the same time.
fn read_env_file(path: &Path) -> Result<StrMap, String> {
    let shown = path.display().to_string();
    dotenvy::from_filename_iter(path)
        .map_err(|e| {
            msgf(
                "runique_test.env_read_failed",
                &[shown.as_str(), e.to_string().as_str()],
            )
        })?
        .map(|item| item.map_err(|e| parse_error(&shown, &e)))
        .collect()
}

// dotenvy's own message quotes the whole broken line, and that line may well
// be `DATABASE_URL=…` with its password.
fn parse_error(path: &str, error: &dotenvy::Error) -> String {
    match error {
        dotenvy::Error::LineParse(_, index) => msgf(
            "runique_test.env_line_malformed",
            &[path, index.to_string().as_str()],
        ),
        other => msgf(
            "runique_test.env_parse_failed",
            &[path, other.to_string().as_str()],
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn absolute_path_is_left_alone() {
        let absolute = std::env::temp_dir().join("x.env");
        assert_eq!(resolve(absolute.to_str().unwrap()), absolute);
    }

    #[test]
    fn timeout_defaults_and_parses() {
        let mut vars = StrMap::new();
        assert_eq!(timeout_from(&vars).unwrap(), DEFAULT_TIMEOUT);
        vars.insert(TIMEOUT_KEY.into(), "3".into());
        assert_eq!(timeout_from(&vars).unwrap(), Duration::from_secs(3));
        for bad in ["0", "-1", "soon", ""] {
            vars.insert(TIMEOUT_KEY.into(), bad.into());
            assert!(timeout_from(&vars).is_err(), "{bad:?} should be refused");
        }
    }
}
