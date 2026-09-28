//! What each `runique_test` run gives back: its result, the SQL it ran, and
//! how that gets printed.
use std::fmt;
use std::io::IsTerminal;
use std::{
    sync::{Arc, Mutex},
    time::Duration,
};

/// One statement a test ran, as caught by the connection's metric callback.
pub struct QueryTrace {
    /// The SQL with placeholders instead of values, so no data leaks to the console.
    pub sql: String,
    pub elapsed: Duration,
    pub failed: bool,
}

/// Where the connection drops every statement it runs.
pub type TraceSink = Arc<Mutex<Vec<QueryTrace>>>;

/// How a test turned out: the handler returned `Ok`, or here's its error.
pub enum Reason<E> {
    Win,
    Error(E),
}

/// Everything a test produced. Its `Display` impl is what gets printed.
pub struct FormatResult<E> {
    pub name_test: String,
    pub reason: Reason<E>,
    pub trace: Vec<QueryTrace>,
    /// Set when the transaction couldn't be rolled back explicitly, with the reason.
    pub rollback_error: Option<String>,
}

impl<E> FormatResult<E> {
    /// A test only passes if its handler succeeded and its transaction got rolled back.
    pub fn passed(&self) -> bool {
        matches!(self.reason, Reason::Win) && self.rollback_error.is_none()
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
            let failed = if query.failed { "  ✗ failed" } else { "" };
            writeln!(
                f,
                "    {step} {:>6.1} ms  {}{failed}",
                query.elapsed.as_secs_f64() * 1000.0,
                shorten_sql(&query.sql)
            )?;
        }
        if let Reason::Error(e) = &self.reason {
            writeln!(f, "    ✗ {e}")?;
        }
        if let Some(e) = &self.rollback_error {
            writeln!(f, "    ✗ rollback: {e}")?;
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
}
