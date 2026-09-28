//! `TestTransaction`: how a database engine opens a throwaway transaction for
//! one test and rolls it back. `ADb` (SeaORM) is the implementation Runique
//! ships; another engine (MongoDB…) can implement the trait on its own client.
use super::struct_test::{QueryTrace, TraceSink};
use crate::db::config::mask_password;
use crate::db::{ADb, DatabaseConfig, RuniqueDb};
use sea_orm::{
    sea_query::{Value, Values},
    {DatabaseConnection, DbErr, Statement, TransactionTrait},
};

#[async_trait::async_trait]
pub trait TestTransaction: Sized + Send + Sync {
    type Config: Send + Sync;
    /// Opened once per `RuniqueTest`, shared by all its tests.
    type Connection: Send + Sync;
    type Error: std::fmt::Display + Send;

    /// Builds the config from the process environment, after the env file was loaded.
    fn load_config() -> Result<Self::Config, Self::Error>;

    /// Human-readable target shown before the run. Must never contain credentials.
    fn describe(config: &Self::Config) -> String;

    /// Connects, and wires every executed statement into `trace`.
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

    fn load_config() -> Result<DatabaseConfig, DbErr> {
        DatabaseConfig::from_env()
            .map(|builder| builder.build())
            .map_err(DbErr::Custom)
    }

    fn describe(config: &DatabaseConfig) -> String {
        format!("{} — {}", config.engine.name(), mask_password(&config.url))
    }

    async fn connect(
        config: &DatabaseConfig,
        trace: TraceSink,
    ) -> Result<DatabaseConnection, DbErr> {
        let mut conn = config.connect().await?;
        // Transactions opened from `conn` inherit this callback, savepoints included.
        conn.set_metric_callback(move |info| {
            trace
                .lock()
                .unwrap_or_else(|e| e.into_inner())
                .push(QueryTrace {
                    sql: mask_hashes(info.statement).to_string(),
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
            Some(RuniqueDb::Conn(_)) => unreachable!("begin_test always opens a transaction"),
            None => Err(DbErr::Custom(
                "a clone of the test connection outlived the handler; \
                 the transaction rolls back when that clone is dropped"
                    .to_string(),
            )),
        }
    }
}

/// Copy of `stmt` whose password-hash values are replaced, so the trace
/// never prints them.
fn mask_hashes(stmt: &Statement) -> Statement {
    let values = stmt.values.as_ref().map(|values| {
        Values(
            values
                .0
                .iter()
                .map(|value| match value {
                    Value::String(Some(s)) if is_password_hash(s) => {
                        Value::String(Some("****".to_string()))
                    }
                    other => other.clone(),
                })
                .collect(),
        )
    });
    Statement {
        sql: stmt.sql.clone(),
        values,
        db_backend: stmt.db_backend,
    }
}

/// PHC/bcrypt prefixes of the hash formats Runique writes (argon2, bcrypt, scrypt).
fn is_password_hash(s: &str) -> bool {
    ["$argon2", "$2a$", "$2b$", "$2y$", "$scrypt$"]
        .iter()
        .any(|prefix| s.starts_with(prefix))
}

#[cfg(test)]
mod tests {
    use super::*;
    use sea_orm::DbBackend;

    #[test]
    fn test_mask_hashes_hides_password_hash_only() {
        let stmt = Statement::from_sql_and_values(
            DbBackend::Postgres,
            "INSERT INTO users (username, password) VALUES ($1, $2)",
            [
                "bob".into(),
                "$argon2id$v=19$m=19456,t=2,p=1$c2FsdA$aGFzaA".into(),
            ],
        );
        let shown = mask_hashes(&stmt).to_string();
        assert!(shown.contains("'bob'"));
        assert!(shown.contains("'****'"));
        assert!(!shown.contains("argon2"));
    }
}
