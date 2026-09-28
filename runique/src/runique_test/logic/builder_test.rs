//! `runique_test`: runs one piece of business logic against a real database,
//! inside a transaction that always gets rolled back, and prints the SQL it ran.
use super::struct_test::{FormatResult, Reason, TraceSink};
use super::transaction_test::TestTransaction;
use crate::utils::aliases::StrMap;
use std::sync::Once;

/// One test at a time across the whole test process: cargo runs tests on
/// several threads, and a SQLite pool only has one connection.
static SERIAL: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());

/// Shows the target database once, not before every single test.
static TARGET_SHOWN: Once = Once::new();

/// Runs `handler` in a fresh transaction on the database `env_file` points to,
/// prints the result and the SQL trace, then rolls back.
///
/// Meant to be called from a `#[tokio::test]`. The handler only borrows the
/// connection, so once it's done this function owns it again and can roll it
/// back. Any failure panics, which is how cargo knows the test failed.
///
/// # Panics
///
/// When the env file can't be read, the database can't be reached, the handler
/// returns an error, or the transaction can't be rolled back.
pub async fn runique_test<C: TestTransaction>(
    env_file: &str,
    handler: impl AsyncFnOnce(&C) -> Result<(), C::Error>,
) {
    let name_test = current_test_name();
    let _serial = SERIAL.lock().await;

    let vars = read_env_file(env_file).unwrap_or_else(|e| panic!("{name_test}: {e}"));
    let config = C::load_config(&vars).unwrap_or_else(|e| panic!("{name_test}: {e}"));
    // Leading newline: cargo has already printed `test … ...` on this line.
    TARGET_SHOWN.call_once(|| println!("\nrunique test → {}", C::describe(&config)));

    let trace = TraceSink::default();
    let connection = C::connect(&config, trace.clone())
        .await
        .unwrap_or_else(|e| panic!("{name_test}: can't connect: {e}"));
    let db = C::begin_test(&connection)
        .await
        .unwrap_or_else(|e| panic!("{name_test}: can't open the test transaction: {e}"));

    let outcome = handler(&db).await;
    let rollback_error = db.rollback_test().await.err().map(|e| e.to_string());

    let result = FormatResult {
        name_test,
        reason: match outcome {
            Ok(()) => Reason::Win,
            Err(e) => Reason::Error(e),
        },
        trace: std::mem::take(&mut *trace.lock().unwrap_or_else(|e| e.into_inner())),
        rollback_error,
    };
    print!("{result}");
    assert!(
        result.passed(),
        "{} failed, see the output above",
        result.name_test
    );
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
        .map_err(|e| format!("can't read {path}: {e}"))?
        .map(|item| item.map_err(|e| format!("can't parse {path}: {e}")))
        .collect()
}
