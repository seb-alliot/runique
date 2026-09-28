//! `runique_test`: runs one piece of business logic against a real database,
//! inside a transaction that always gets rolled back, and prints the SQL it ran.
use super::struct_test::{FormatResult, Reason, TestFailure, TraceSink, msg, msgf};
use super::transaction_test::TestTransaction;
use crate::utils::aliases::StrMap;
use futures_util::FutureExt;
use std::any::Any;
use std::panic::AssertUnwindSafe;
use std::sync::Once;

/// One test at a time across the whole test process: cargo runs tests on
/// several threads, and a SQLite pool only has one connection.
static SERIAL: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());

/// Shows the target database once, not before every single test.
static TARGET_SHOWN: Once = Once::new();

/// Runs `handler` in a fresh transaction on the database `env_file` points to,
/// rolls back, then prints the result and the SQL trace.
///
/// Meant to be the last expression of a `#[tokio::test]` that returns
/// `Result<(), TestFailure>`. The handler only borrows the connection, so once
/// it's done this function owns it again and can roll it back.
///
/// Never panics: a handler that errors or panics still gets its transaction
/// rolled back, and every failure (setup included) comes back as `Err`.
pub async fn runique_test<C: TestTransaction>(
    env_file: &str,
    handler: impl AsyncFnOnce(&C) -> Result<(), C::Error>,
) -> Result<(), TestFailure> {
    let name_test = current_test_name();
    let _serial = SERIAL.lock().await;
    let trace = TraceSink::default();

    let (reason, rollback_error) = match open::<C>(env_file, &trace).await {
        Err(setup) => (Reason::Setup(setup), None),
        Ok((_connection, db)) => {
            // After a panic, the only thing still used is the transaction, and
            // all that happens to it is a rollback.
            let outcome = AssertUnwindSafe(handler(&db)).catch_unwind().await;
            let rollback_error = db.rollback_test().await.err().map(|e| e.to_string());
            let reason = match outcome {
                Ok(Ok(())) => Reason::Win,
                Ok(Err(e)) => Reason::Error(e),
                Err(payload) => Reason::Panic(panic_message(payload.as_ref())),
            };
            (reason, rollback_error)
        }
    };

    let result = FormatResult {
        name_test,
        reason,
        trace: std::mem::take(&mut *trace.lock().unwrap_or_else(|e| e.into_inner())),
        rollback_error,
    };
    print!("{result}");
    if result.passed() {
        Ok(())
    } else {
        Err(TestFailure {
            name_test: result.name_test,
        })
    }
}

/// Everything before the handler runs. The connection is handed back too,
/// so it outlives the transaction opened on it.
async fn open<C: TestTransaction>(
    env_file: &str,
    trace: &TraceSink,
) -> Result<(C::Connection, C), String> {
    let vars = read_env_file(env_file)?;
    let config = C::load_config(&vars).map_err(|e| e.to_string())?;
    // Leading newline: cargo has already printed `test … ...` on this line.
    TARGET_SHOWN.call_once(|| println!("\nrunique test → {}", C::describe(&config)));

    let connection = C::connect(&config, trace.clone())
        .await
        .map_err(|e| msgf("runique_test.connect_failed", &[e]))?;
    let db = C::begin_test(&connection)
        .await
        .map_err(|e| msgf("runique_test.begin_failed", &[e]))?;
    Ok((connection, db))
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
fn read_env_file(path: &str) -> Result<StrMap, String> {
    dotenvy::from_filename_iter(path)
        .map_err(|e| {
            msgf(
                "runique_test.env_read_failed",
                &[path, e.to_string().as_str()],
            )
        })?
        .map(|item| item.map_err(|e| parse_error(path, &e)))
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
