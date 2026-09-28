//! `ADb`: the framework's database handle, the type behind every `&ADb`
//! parameter in the framework (`search!`, forms, admin, auth).
//!
//! It wraps either a real [`DatabaseConnection`] or, with the `test-utils`
//! feature, a [`RuniqueDb`](crate::db::RuniqueDb), behind an `Arc`. Unlike a
//! bare `Arc<T>`, `ADb` implements `ConnectionTrait`/`TransactionTrait`
//! itself. `Arc<T>` can't get those impls from this crate: `Arc` isn't
//! `#[fundamental]`, so the orphan rule rules it out. And that's the whole
//! point: every native SeaORM call (`.insert(db)`, `.one(db)`, `db.begin()`, …)
//! takes `&ADb` as is, no `.as_ref()` needed.
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

/// Thread-safe database handle that's cheap to clone. See the module docs.
#[derive(Clone, Debug)]
pub struct ADb(Arc<Inner>);

impl ADb {
    pub fn new(inner: Inner) -> Self {
        Self(Arc::new(inner))
    }

    /// Builds an `ADb` from a freshly opened [`DatabaseConnection`], which is
    /// what you want most of the time (boot, CLI tools, test setup). With
    /// `test-utils` on, it also does the `RuniqueDb::Conn` wrapping for you, so
    /// callers never have to care that `Inner` changes with the feature.
    #[cfg(not(feature = "test-utils"))]
    pub fn from_connection(conn: sea_orm::DatabaseConnection) -> Self {
        Self::new(conn)
    }

    /// Same as the `not(test-utils)` version above.
    #[cfg(feature = "test-utils")]
    pub fn from_connection(conn: sea_orm::DatabaseConnection) -> Self {
        Self::new(crate::db::RuniqueDb::Conn(conn))
    }

    /// Hands back the inner handle, or `None` if another clone is still around.
    /// The test builder has to own the transaction to roll it back, since
    /// `DatabaseTransaction::rollback` takes `self`.
    #[cfg(feature = "test-utils")]
    pub(crate) fn into_inner(self) -> Option<Inner> {
        Arc::into_inner(self.0)
    }

    /// The same shortcut `DatabaseConnection` has. Without it,
    /// `db.get_database_backend()` would only resolve through `ConnectionTrait`,
    /// so callers would have to import the trait for this one call: method
    /// lookup stops at the first match and never falls through to `Inner`'s
    /// own inherent method.
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

// `Transaction = DatabaseTransaction` either way: `DatabaseConnection` and
// `RuniqueDb` already use that type in their own impls, so each method here is a
// plain one-line delegation, with no recursive nesting to worry about.
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
