//! `TestTransaction`: how a database engine opens a throwaway transaction for
//! a test and rolls it back afterwards. Runique ships the `ADb` (SeaORM)
//! implementation; any other engine (MongoDB…) can implement it for its own client.
use super::struct_test::{QueryTrace, TraceSink, msg};
use crate::admin::helper::text_cast_type;
use crate::db::config::mask_password;
use crate::db::{ADb, DatabaseConfig, RuniqueDb};
use crate::utils::aliases::StrMap;
use sea_orm::{ConnectionTrait, DatabaseConnection, DbBackend, DbErr, TransactionTrait};
use sea_query::{Alias, Expr, ExprTrait, Func, Query};
use std::time::{Duration, Instant};

/// How long `rollback_test` waits for a clone of the test connection (kept
/// by a task the handler spawned) to be dropped before giving up.
const CLONE_GRACE: Duration = Duration::from_secs(3);

#[async_trait::async_trait]
pub trait TestTransaction: Sized + Send + Sync {
    type Config: Send + Sync;
    /// Opened for a single test, then dropped.
    type Connection: Send + Sync;
    type Error: std::fmt::Display + Send;

    /// Builds the config from the values of the env file the test named, and
    /// only from them: nothing is read from the shell's environment.
    fn load_config(vars: &StrMap) -> Result<Self::Config, Self::Error>;

    /// A readable description of the target, shown before the run. Never put credentials in it.
    fn describe(config: &Self::Config) -> String;

    /// Connects, and hooks every statement it runs into `trace`.
    async fn connect(
        config: &Self::Config,
        trace: TraceSink,
    ) -> Result<Self::Connection, Self::Error>;

    async fn begin_test(conn: &Self::Connection) -> Result<Self, Self::Error>;

    /// Whether the test transaction is still the one `begin_test` opened, after
    /// the handler. `false` when the handler ended it (a `COMMIT`, or DDL on an
    /// engine that commits implicitly): the final rollback can't undo anything
    /// then, and some engines don't even report it. Answer `true` when the
    /// engine can't tell.
    async fn still_open(&self) -> Result<bool, Self::Error> {
        Ok(true)
    }

    async fn rollback_test(self) -> Result<(), Self::Error>;
}

#[async_trait::async_trait]
impl TestTransaction for ADb {
    type Config = DatabaseConfig;
    type Connection = DatabaseConnection;
    type Error = DbErr;

    fn load_config(vars: &StrMap) -> Result<DatabaseConfig, DbErr> {
        // Without either key, the config would quietly fall back to a local
        // SQLite file: the test database has to be named on purpose.
        if !vars.contains_key("DATABASE_URL") && !vars.contains_key("DB_ENGINE") {
            return Err(DbErr::Custom(msg("runique_test.no_db_config").into_owned()));
        }
        // The file only: a key missing from it must not be filled in from the
        // shell, where it could point anywhere (production included).
        let mut config = DatabaseConfig::from_lookup(|key| vars.get(key).cloned())
            .map(|builder| builder.build())
            .map_err(DbErr::Custom)?;
        // A test only ever needs the one connection its transaction runs on.
        config.max_connections = 1;
        config.min_connections = 1;
        Ok(config)
    }

    fn describe(config: &DatabaseConfig) -> String {
        let target = format!("{} — {}", config.engine.name(), mask_password(&config.url));
        if config.url.contains(":memory:") || config.url.contains("mode=memory") {
            format!("{target} ({})", msg("runique_test.in_memory"))
        } else {
            target
        }
    }

    async fn connect(
        config: &DatabaseConfig,
        trace: TraceSink,
    ) -> Result<DatabaseConnection, DbErr> {
        let mut conn = config.connect().await?;
        // Transactions opened from `conn` pick up this callback too, savepoints included.
        conn.set_metric_callback(move |info| {
            trace
                .lock()
                .unwrap_or_else(|e| e.into_inner())
                .push(QueryTrace {
                    // Placeholders only (`$1`, `?`) for queries SeaORM builds.
                    // SQL written by hand with its values inlined shows them.
                    sql: info.statement.sql.clone(),
                    elapsed: info.elapsed,
                    failed: info.failed,
                    expected: false,
                });
        });
        Ok(conn)
    }

    async fn begin_test(conn: &DatabaseConnection) -> Result<Self, DbErr> {
        Ok(ADb::new(RuniqueDb::Txn(conn.begin().await?)))
    }

    async fn still_open(&self) -> Result<bool, DbErr> {
        match self.get_database_backend() {
            // Inside a transaction, the id stays the same from one statement to
            // the next; once it's committed, every statement gets a new one.
            DbBackend::Postgres => {
                let txid = || Expr::expr(Func::cust(Alias::new("txid_current")));
                let first = scalar(self, txid()).await;
                let second = scalar(self, txid()).await;
                match (first, second) {
                    (Ok(a), Ok(b)) => Ok(a == b),
                    // Only a transaction left aborted by a failed statement
                    // refuses these, and that one is still open.
                    _ => Ok(true),
                }
            }
            // MariaDB has the variable; MySQL doesn't, and then there's no way to tell.
            // sea-query has no system variable, `custom_keyword` is its documented way out.
            DbBackend::MySql => {
                match scalar(self, Expr::custom_keyword(Alias::new("@@in_transaction"))).await {
                    Ok(v) => Ok(v != "0"),
                    Err(_) => Ok(true),
                }
            }
            // SQLite already refuses to roll back a transaction that's gone.
            _ => Ok(true),
        }
    }

    async fn rollback_test(self) -> Result<(), DbErr> {
        // A task the handler spawned may still hold a clone for a moment
        // (sending a mail, say); give it a chance to finish before giving up.
        let deadline = Instant::now() + CLONE_GRACE;
        while self.is_shared() && Instant::now() < deadline {
            tokio::time::sleep(Duration::from_millis(20)).await;
        }
        match self.into_inner() {
            Some(RuniqueDb::Txn(txn)) => txn.rollback().await,
            // Can't happen, since `begin_test` always opens one; still an error
            // rather than a panic, because this runs outside the panic guard.
            Some(RuniqueDb::Conn(_)) => Err(DbErr::Custom(
                msg("runique_test.not_a_transaction").into_owned(),
            )),
            None => Err(DbErr::Custom(
                msg("runique_test.clone_outlived").into_owned(),
            )),
        }
    }
}

/// `SELECT expr`, read back as text.
async fn scalar(db: &ADb, expr: Expr) -> Result<String, DbErr> {
    let stmt = Query::select()
        .expr(expr.cast_as(Alias::new(text_cast_type(db))))
        .to_owned();
    db.query_one(&stmt)
        .await?
        .ok_or_else(|| DbErr::RecordNotFound("SELECT".into()))?
        .try_get_by_index::<String>(0)
}

/// Written from cargo-mutants survivors (2026-10-08): the target line the
/// builder prints names the engine, never the password.
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_target_names_the_engine_without_the_password() {
        let memory = DatabaseConfig::from_url("sqlite://test.db?mode=memory")
            .unwrap()
            .build();
        let shown = <ADb as TestTransaction>::describe(&memory);
        assert!(shown.contains("SQLite"), "{shown}");
        assert!(shown.contains(&*msg("runique_test.in_memory")), "{shown}");

        let remote = DatabaseConfig::from_url("postgres://u:secret@db.example/app")
            .unwrap()
            .build();
        let shown = <ADb as TestTransaction>::describe(&remote);
        assert!(
            shown.contains("db.example") && !shown.contains("secret"),
            "{shown}"
        );
        assert!(!shown.contains(&*msg("runique_test.in_memory")), "{shown}");
    }
}
