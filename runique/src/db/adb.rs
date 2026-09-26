//! `ADb`: the framework's database handle — the type behind every `&ADb`
//! signature across the framework (`search!`, forms, admin, auth).
//!
//! Wraps either a real [`DatabaseConnection`], or (under the `test-utils`
//! feature) [`RuniqueDb`](crate::db::RuniqueDb), behind an `Arc`. Unlike a
//! bare `Arc<T>`, `ADb` implements `ConnectionTrait`/`TransactionTrait`
//! **directly** — `Arc<T>` cannot (`Arc` isn't `#[fundamental]`, so a
//! blanket impl from this crate would violate the orphan rule). That direct
//! impl is the whole point: every native SeaORM call (`.insert(db)`,
//! `.one(db)`, `db.begin()`, …) accepts `&ADb` with no `.as_ref()` needed.
use sea_orm::{
    AccessMode, ConnectionTrait, DatabaseTransaction, DbBackend, DbErr, ExecResult, IsolationLevel,
    QueryResult, Statement, TransactionError, TransactionOptions, TransactionTrait,
};
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

#[cfg(not(feature = "test-utils"))]
type Inner = sea_orm::DatabaseConnection;
#[cfg(feature = "test-utils")]
type Inner = crate::db::RuniqueDb;

/// Thread-safe, cheaply-cloneable database handle — see module docs.
#[derive(Clone, Debug)]
pub struct ADb(Arc<Inner>);

impl ADb {
    pub fn new(inner: Inner) -> Self {
        Self(Arc::new(inner))
    }

    /// Builds an `ADb` from a freshly-opened [`DatabaseConnection`] — the
    /// common case (boot, CLI tools, test setup). Absorbs the
    /// `RuniqueDb::Conn` wrap internally when `test-utils` is active, so
    /// callers never need to know `Inner` differs per feature.
    #[cfg(not(feature = "test-utils"))]
    pub fn from_connection(conn: sea_orm::DatabaseConnection) -> Self {
        Self::new(conn)
    }

    /// See the `not(test-utils)` variant above.
    #[cfg(feature = "test-utils")]
    pub fn from_connection(conn: sea_orm::DatabaseConnection) -> Self {
        Self::new(crate::db::RuniqueDb::Conn(conn))
    }

    /// Inherent shortcut mirroring `DatabaseConnection`'s own inherent method
    /// of the same name — without it, `db.get_database_backend()` would only
    /// resolve through `ConnectionTrait`, requiring callers to import it just
    /// for this one call (`ConnectionTrait` method resolution doesn't fall
    /// through to `Inner`'s own inherent method past the first match).
    pub fn get_database_backend(&self) -> DbBackend {
        ConnectionTrait::get_database_backend(self)
    }
}

impl std::ops::Deref for ADb {
    type Target = Inner;
    fn deref(&self) -> &Inner {
        &self.0
    }
}

impl From<Inner> for ADb {
    fn from(inner: Inner) -> Self {
        Self::new(inner)
    }
}

#[async_trait::async_trait]
impl ConnectionTrait for ADb {
    fn get_database_backend(&self) -> DbBackend {
        self.0.get_database_backend()
    }

    async fn execute_raw(&self, stmt: Statement) -> Result<ExecResult, DbErr> {
        self.0.execute_raw(stmt).await
    }

    async fn execute_unprepared(&self, sql: &str) -> Result<ExecResult, DbErr> {
        self.0.execute_unprepared(sql).await
    }

    async fn query_one_raw(&self, stmt: Statement) -> Result<Option<QueryResult>, DbErr> {
        self.0.query_one_raw(stmt).await
    }

    async fn query_all_raw(&self, stmt: Statement) -> Result<Vec<QueryResult>, DbErr> {
        self.0.query_all_raw(stmt).await
    }
}

// `Transaction = DatabaseTransaction` in both configurations — `DatabaseConnection`
// and `RuniqueDb` already resolve to that same associated type on their own impl,
// so this is a pure one-line delegation per method, never recursive nesting.
#[async_trait::async_trait]
impl TransactionTrait for ADb {
    type Transaction = DatabaseTransaction;

    async fn begin(&self) -> Result<Self::Transaction, DbErr> {
        self.0.begin().await
    }

    async fn begin_with_config(
        &self,
        isolation_level: Option<IsolationLevel>,
        access_mode: Option<AccessMode>,
    ) -> Result<Self::Transaction, DbErr> {
        self.0.begin_with_config(isolation_level, access_mode).await
    }

    async fn begin_with_options(
        &self,
        options: TransactionOptions,
    ) -> Result<Self::Transaction, DbErr> {
        self.0.begin_with_options(options).await
    }

    async fn transaction<F, T, E>(&self, callback: F) -> Result<T, TransactionError<E>>
    where
        F: for<'c> FnOnce(
                &'c Self::Transaction,
            ) -> Pin<Box<dyn Future<Output = Result<T, E>> + Send + 'c>>
            + Send,
        T: Send,
        E: std::fmt::Display + std::fmt::Debug + Send,
    {
        self.0.transaction(callback).await
    }

    async fn transaction_with_config<F, T, E>(
        &self,
        callback: F,
        isolation_level: Option<IsolationLevel>,
        access_mode: Option<AccessMode>,
    ) -> Result<T, TransactionError<E>>
    where
        F: for<'c> FnOnce(
                &'c Self::Transaction,
            ) -> Pin<Box<dyn Future<Output = Result<T, E>> + Send + 'c>>
            + Send,
        T: Send,
        E: std::fmt::Display + std::fmt::Debug + Send,
    {
        self.0
            .transaction_with_config(callback, isolation_level, access_mode)
            .await
    }
}
