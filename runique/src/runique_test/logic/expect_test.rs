//! `expect_db_error`: checks that the database refuses something, without the
//! refusal counting as an error the test swallowed.
use super::builder_test::{mark_expected_from, trace_len};
use super::struct_test::msg;
use crate::db::ADb;
use sea_orm::{DatabaseTransaction, DbErr, TransactionTrait};

/// Runs `action` in a savepoint that always gets rolled back, and expects the
/// database to refuse it: a unique constraint, a foreign key, a `NOT NULL`…
///
/// Returns the database's error when it refused, and an `Err` when it
/// accepted. The savepoint keeps the rest of the test going: on Postgres, a
/// failed statement outside one spoils the whole transaction. Failures in
/// here are marked as expected in the trace, so they don't fail the test.
///
/// ```rust,ignore
/// let refused = expect_db_error(db, async |sp| {
///     contribution(deleted_user_id).insert(sp).await
/// })
/// .await?;
/// ```
pub async fn expect_db_error<T>(
    db: &ADb,
    action: impl AsyncFnOnce(&DatabaseTransaction) -> Result<T, DbErr>,
) -> Result<DbErr, DbErr> {
    let start = trace_len();
    let savepoint = db.begin().await?;
    let attempt = action(&savepoint).await;
    savepoint.rollback().await?;
    mark_expected_from(start);
    match attempt {
        Err(refused) => Ok(refused),
        Ok(_) => Err(DbErr::Custom(msg("runique_test.not_refused").into_owned())),
    }
}
