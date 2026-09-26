//! `RuniqueDb`: a database handle that is either a real connection or an
//! already-open transaction — the type behind [`ADb`](crate::utils::aliases::ADb)
//! when the `test-utils` feature is enabled.
//!
//! The whole point: every function written against `&ADb` (the framework's
//! entire DB-facing surface — `search!`, forms, admin) works unchanged
//! whether it's handed a real connection or a transaction opened by the test
//! harness for one test body, rolled back at the end (Django `TestCase`
//! style) — no `#[serial]`, no shared-DB test isolation bugs.
#![cfg(feature = "test-utils")]

use sea_orm::{
    AccessMode, ConnectionTrait, DatabaseConnection, DatabaseTransaction, DbBackend, DbErr,
    ExecResult, IsolationLevel, QueryResult, Statement, TransactionError, TransactionOptions,
    TransactionTrait,
};
use std::future::Future;
use std::pin::Pin;

/// Either a real [`DatabaseConnection`], or a [`DatabaseTransaction`] already
/// open (used by the test harness to wrap one test's body — nothing it
/// writes is ever actually committed).
#[derive(Debug)]
pub enum RuniqueDb {
    Conn(DatabaseConnection),
    Txn(DatabaseTransaction),
}

#[async_trait::async_trait]
impl ConnectionTrait for RuniqueDb {
    fn get_database_backend(&self) -> DbBackend {
        match self {
            Self::Conn(c) => c.get_database_backend(),
            Self::Txn(t) => t.get_database_backend(),
        }
    }

    async fn execute_raw(&self, stmt: Statement) -> Result<ExecResult, DbErr> {
        match self {
            Self::Conn(c) => c.execute_raw(stmt).await,
            Self::Txn(t) => t.execute_raw(stmt).await,
        }
    }

    async fn execute_unprepared(&self, sql: &str) -> Result<ExecResult, DbErr> {
        match self {
            Self::Conn(c) => c.execute_unprepared(sql).await,
            Self::Txn(t) => t.execute_unprepared(sql).await,
        }
    }

    async fn query_one_raw(&self, stmt: Statement) -> Result<Option<QueryResult>, DbErr> {
        match self {
            Self::Conn(c) => c.query_one_raw(stmt).await,
            Self::Txn(t) => t.query_one_raw(stmt).await,
        }
    }

    async fn query_all_raw(&self, stmt: Statement) -> Result<Vec<QueryResult>, DbErr> {
        match self {
            Self::Conn(c) => c.query_all_raw(stmt).await,
            Self::Txn(t) => t.query_all_raw(stmt).await,
        }
    }
}

// `Transaction = DatabaseTransaction` on purpose in both branches — never
// `Self` — see the module doc. `Conn(c).begin()` opens a normal transaction;
// `Txn(t).begin()` opens a savepoint. Both are already a real
// `DatabaseTransaction` on their own, which already satisfies
// `TransactionTrait<Transaction = Self::Transaction>` by itself, so nothing
// here has to re-implement the recursive nesting.
#[async_trait::async_trait]
impl TransactionTrait for RuniqueDb {
    type Transaction = DatabaseTransaction;

    async fn begin(&self) -> Result<Self::Transaction, DbErr> {
        match self {
            Self::Conn(c) => c.begin().await,
            Self::Txn(t) => t.begin().await,
        }
    }

    async fn begin_with_config(
        &self,
        isolation_level: Option<IsolationLevel>,
        access_mode: Option<AccessMode>,
    ) -> Result<Self::Transaction, DbErr> {
        match self {
            Self::Conn(c) => c.begin_with_config(isolation_level, access_mode).await,
            Self::Txn(t) => t.begin_with_config(isolation_level, access_mode).await,
        }
    }

    async fn begin_with_options(
        &self,
        options: TransactionOptions,
    ) -> Result<Self::Transaction, DbErr> {
        match self {
            Self::Conn(c) => c.begin_with_options(options).await,
            Self::Txn(t) => t.begin_with_options(options).await,
        }
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
        match self {
            Self::Conn(c) => c.transaction(callback).await,
            Self::Txn(t) => t.transaction(callback).await,
        }
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
        match self {
            Self::Conn(c) => {
                c.transaction_with_config(callback, isolation_level, access_mode)
                    .await
            }
            Self::Txn(t) => {
                t.transaction_with_config(callback, isolation_level, access_mode)
                    .await
            }
        }
    }
}
