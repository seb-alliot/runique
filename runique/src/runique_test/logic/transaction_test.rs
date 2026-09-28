//! `TestTransaction`: how a database engine opens a throwaway transaction for
//! a test and rolls it back afterwards. Runique ships the `ADb` (SeaORM)
//! implementation; any other engine (MongoDB…) can implement it for its own client.
use super::struct_test::{QueryTrace, TraceSink, msg};
use crate::db::config::mask_password;
use crate::db::{ADb, DatabaseConfig, RuniqueDb};
use crate::utils::aliases::StrMap;
use sea_orm::{DatabaseConnection, DbErr, TransactionTrait};

#[async_trait::async_trait]
pub trait TestTransaction: Sized + Send + Sync {
    type Config: Send + Sync;
    /// Opened for a single test, then dropped.
    type Connection: Send + Sync;
    type Error: std::fmt::Display + Send;

    /// Builds the config from the values of the env file the test named.
    fn load_config(vars: &StrMap) -> Result<Self::Config, Self::Error>;

    /// A readable description of the target, shown before the run. Never put credentials in it.
    fn describe(config: &Self::Config) -> String;

    /// Connects, and hooks every statement it runs into `trace`.
    async fn connect(
        config: &Self::Config,
        trace: TraceSink,
    ) -> Result<Self::Connection, Self::Error>;

    async fn begin_test(conn: &Self::Connection) -> Result<Self, Self::Error>;

    async fn rollback_test(self) -> Result<(), Self::Error>;
}

#[async_trait::async_trait]
impl TestTransaction for ADb {
    type Config = DatabaseConfig;
    type Connection = DatabaseConnection;
    type Error = DbErr;

    fn load_config(vars: &StrMap) -> Result<DatabaseConfig, DbErr> {
        // The file wins; a key it doesn't set falls back to the environment,
        // which is only read here, never written.
        let mut config = DatabaseConfig::from_lookup(|key| {
            vars.get(key).cloned().or_else(|| std::env::var(key).ok())
        })
        .map(|builder| builder.build())
        .map_err(DbErr::Custom)?;
        // A test only ever needs the one connection its transaction runs on.
        config.max_connections = 1;
        config.min_connections = 1;
        Ok(config)
    }

    fn describe(config: &DatabaseConfig) -> String {
        format!("{} — {}", config.engine.name(), mask_password(&config.url))
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
                    // Placeholders only (`$1`, `?`): no value from the database
                    // ever reaches the console.
                    sql: info.statement.sql.clone(),
                    elapsed: info.elapsed,
                    failed: info.failed,
                });
        });
        Ok(conn)
    }

    async fn begin_test(conn: &DatabaseConnection) -> Result<Self, DbErr> {
        Ok(ADb::new(RuniqueDb::Txn(conn.begin().await?)))
    }

    async fn rollback_test(self) -> Result<(), DbErr> {
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
