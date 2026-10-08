//! What each `runique_test` run gives back: its result, the SQL it ran, and
//! how that gets printed.
use crate::utils::trad::Lang;
use std::borrow::Cow;
use std::fmt;
use std::io::IsTerminal;
use std::{
    sync::{Arc, LazyLock, Mutex},
    time::Duration,
};

/// The language everything the test builder prints is in, from the user's
/// locale. Not `set_lang`: that one is process-wide, and the app's other tests
/// share this binary and may count on the default language.
static LANG: LazyLock<Lang> = LazyLock::new(|| Lang::from_env().unwrap_or_default());

pub(super) fn msg(key: &str) -> Cow<'static, str> {
    LANG.get(key)
}

pub(super) fn msgf<T: fmt::Display>(key: &str, args: &[T]) -> String {
    LANG.format(key, args)
}

/// One statement a test ran, as caught by the connection's metric callback
/// (or by `ADb` itself for `execute_unprepared`, which SeaORM doesn't report).
pub struct QueryTrace {
    /// The SQL with placeholders instead of values, so no data leaks to the console.
    pub sql: String,
    pub elapsed: Duration,
    pub failed: bool,
    /// Ran inside `expect_db_error`: a failure here is what the test asked for.
    pub expected: bool,
}

/// Where the connection drops every statement it runs.
pub type TraceSink = Arc<Mutex<Vec<QueryTrace>>>;

/// How the handler turned out.
pub enum Reason<E> {
    /// The handler returned `Ok`.
    Win,
    /// The handler returned this error.
    Error(E),
    /// The handler panicked, with this message. The transaction still got rolled back.
    Panic(String),
    /// The handler was still running when the time limit ran out, in seconds.
    TimedOut(u64),
    /// The test never reached its handler: env file, config, connection or transaction.
    Setup(String),
}

/// Something the builder caught on its own, whatever the handler returned.
pub enum Issue {
    /// The transaction was ended from inside the test (a `COMMIT`, or DDL on
    /// MariaDB/MySQL): what the test wrote before that is now in the database.
    TransactionEnded,
    /// Queries failed but the handler still returned `Ok`: an error got
    /// swallowed somewhere. Failures inside `expect_db_error` don't count.
    SwallowedFailures(usize),
    /// The transaction couldn't be rolled back explicitly, with the reason.
    RollbackFailed(String),
}

impl fmt::Display for Issue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Issue::TransactionEnded => f.write_str(&msg("runique_test.transaction_ended")),
            Issue::SwallowedFailures(n) => {
                f.write_str(&msgf("runique_test.swallowed_failures", &[n]))
            }
            Issue::RollbackFailed(e) => f.write_str(&msgf("runique_test.rollback_failed", &[e])),
        }
    }
}

/// What a failed `runique_test` hands back to cargo. The details are already
/// printed by then; `message` repeats the main one so a test can check it.
pub struct TestFailure {
    pub name_test: String,
    pub message: String,
}

// Cargo shows a failing test's `Err` through `Debug`.
impl fmt::Debug for TestFailure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&msgf("runique_test.test_failed", &[&self.name_test]))
    }
}

impl fmt::Display for TestFailure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}

impl std::error::Error for TestFailure {}

/// Everything a test produced. Its `Display` impl is what gets printed.
pub struct FormatResult<E> {
    pub name_test: String,
    pub reason: Reason<E>,
    pub trace: Vec<QueryTrace>,
    pub issues: Vec<Issue>,
}

impl<E: fmt::Display> FormatResult<E> {
    /// A test only passes if its handler succeeded and the builder caught nothing.
    pub fn passed(&self) -> bool {
        matches!(self.reason, Reason::Win) && self.issues.is_empty()
    }

    /// The first thing that went wrong, as printed. `None` for a test that passed.
    pub fn failure_message(&self) -> Option<String> {
        reason_message(&self.reason).or_else(|| self.issues.first().map(Issue::to_string))
    }
}

fn reason_message<E: fmt::Display>(reason: &Reason<E>) -> Option<String> {
    match reason {
        Reason::Win => None,
        Reason::Error(e) => Some(e.to_string()),
        Reason::Panic(message) => Some(msgf("runique_test.panicked", &[message])),
        Reason::TimedOut(secs) => Some(msgf("runique_test.timed_out", &[secs])),
        Reason::Setup(message) => Some(message.clone()),
    }
}

// Cargo already prints each test's name and its `ok`/`FAILED`, so this only
// adds the SQL trace and, when something went wrong, why.
impl<E: fmt::Display> fmt::Display for FormatResult<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Ends cargo's `test … ...` line so the trace starts on a line of its own.
        writeln!(f)?;
        let color = use_color();
        for (n, query) in (1..).zip(&self.trace) {
            let step = if color {
                format!("\x1b[1;36m{n:>2}.\x1b[0m")
            } else {
                format!("{n:>2}.")
            };
            let failed = match (query.failed, query.expected) {
                (false, _) => String::new(),
                (true, false) => format!("  ✗ {}", msg("runique_test.query_failed")),
                (true, true) => format!(
                    "  ✗ {} ({})",
                    msg("runique_test.query_failed"),
                    msg("runique_test.expected")
                ),
            };
            writeln!(
                f,
                "    {step} {:>6.1} ms  {}{failed}",
                query.elapsed.as_secs_f64() * 1000.0,
                shorten_sql(&query.sql)
            )?;
        }
        if let Some(message) = reason_message(&self.reason) {
            writeln!(f, "    ✗ {message}")?;
        }
        for issue in &self.issues {
            writeln!(f, "    ✗ {issue}")?;
        }
        Ok(())
    }
}

/// Colors only when someone is reading: not in CI logs or piped output, and
/// never when `NO_COLOR` is set. `runique test` pipes the output to space it
/// out, so it tells us through an env var that a terminal is on the other end.
fn use_color() -> bool {
    std::env::var_os("NO_COLOR").is_none()
        && (std::env::var_os(crate::cli::test_runner::COLOR_ENV).is_some()
            || std::io::stdout().is_terminal())
}

/// Drops what makes a query long without telling you anything: a `SELECT`'s
/// column list (shown as `…`) and a write's `RETURNING` clause.
fn shorten_sql(sql: &str) -> String {
    let sql = sql.split_once(" RETURNING ").map_or(sql, |(head, _)| head);
    match (sql.starts_with("SELECT "), sql.find(" FROM ")) {
        (true, Some(from)) => format!("SELECT …{}", &sql[from..]),
        _ => sql.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::shorten_sql;

    #[test]
    fn select_column_list_is_collapsed() {
        let sql =
            r#"SELECT "blog"."id", "blog"."title" FROM "blog" WHERE "blog"."id" = $1 LIMIT $2"#;
        assert_eq!(
            shorten_sql(sql),
            r#"SELECT … FROM "blog" WHERE "blog"."id" = $1 LIMIT $2"#
        );
    }

    #[test]
    fn returning_clause_is_dropped() {
        let sql = r#"INSERT INTO "blog" ("title") VALUES ($1) RETURNING "id", "title""#;
        assert_eq!(
            shorten_sql(sql),
            r#"INSERT INTO "blog" ("title") VALUES ($1)"#
        );
    }

    #[test]
    fn other_statements_are_left_alone() {
        let sql = r#"UPDATE "blog" SET "title" = $1 WHERE "blog"."id" = $2"#;
        assert_eq!(shorten_sql(sql), sql);
    }

    // Written from cargo-mutants survivors (2026-10-08): what the builder
    // prints is the translation, with its arguments.
    #[test]
    fn messages_are_translated_with_their_arguments() {
        use super::{Issue, TestFailure, msg, msgf};
        let key = "runique_test.transaction_ended";
        let lang = crate::utils::trad::Lang::from_env().unwrap_or_default();
        assert_eq!(msg(key), lang.get(key));
        assert_ne!(msg(key), key);
        assert_eq!(Issue::TransactionEnded.to_string(), msg(key));
        assert!(Issue::SwallowedFailures(3).to_string().contains('3'));
        assert!(
            Issue::RollbackFailed("lost".into())
                .to_string()
                .contains("lost")
        );
        let failure = TestFailure {
            name_test: "user::add_email".into(),
            message: String::new(),
        };
        assert!(format!("{failure:?}").contains("user::add_email"));
        assert_eq!(failure.to_string(), format!("{failure:?}"));
        assert!(msgf("runique_test.swallowed_failures", &[7]).contains('7'));
    }
}
