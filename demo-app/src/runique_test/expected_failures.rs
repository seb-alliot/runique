//! Tests that make `runique_test` fail on purpose: the failure shows up in the
//! output, yet the test itself passes, because what's checked is that the
//! failure was reported and nothing stayed in the database.
use crate::backend::auth::find_user_by_username;
use runique::prelude::runique_users::ActiveModel as UserActiveModel;
use runique::prelude::*;
use runique::runique_test::{TestFailure, runique_test};
use sea_orm::{ConnectionTrait, DbErr, Statement};

const ERROR_USER: &str = "runique_test_expected_error";
const PANIC_USER: &str = "runique_test_expected_panic";
const QUERY_USER: &str = "runique_test_expected_bad_query";

fn new_user(username: &str) -> UserActiveModel {
    UserActiveModel {
        username: Set(username.to_string()),
        email: Set(format!("{username}@example.com")),
        password: Set("not-a-real-hash".to_string()),
        is_active: Set(false),
        is_staff: Set(false),
        is_superuser: Set(false),
        ..Default::default()
    }
}

/// Opens a transaction of its own, so it only sees what actually got committed.
async fn is_rolled_back(username: &str) -> Result<(), TestFailure> {
    runique_test::<ADb>(super::ENV, async |db| {
        match find_user_by_username(db, username).await {
            None => Ok(()),
            Some(_) => Err(DbErr::Custom(format!("{username} survived the rollback"))),
        }
    })
    .await
}

#[tokio::test]
async fn handler_error_is_reported_and_rolled_back() -> Result<(), TestFailure> {
    let outcome = runique_test::<ADb>(super::ENV, async |db| {
        new_user(ERROR_USER).insert(db).await?;
        Err(DbErr::Custom("this test fails on purpose".into()))
    })
    .await;
    assert!(outcome.is_err(), "a handler error must fail the test");
    is_rolled_back(ERROR_USER).await
}

#[tokio::test]
async fn handler_panic_is_caught_and_rolled_back() -> Result<(), TestFailure> {
    let outcome = runique_test::<ADb>(super::ENV, async |db| {
        new_user(PANIC_USER).insert(db).await?;
        panic!("this test panics on purpose");
    })
    .await;
    assert!(outcome.is_err(), "a handler panic must fail the test");
    // Also proves the panic didn't leave the serial lock held.
    is_rolled_back(PANIC_USER).await
}

#[tokio::test]
async fn failed_query_is_marked_and_rolled_back() -> Result<(), TestFailure> {
    let outcome = runique_test::<ADb>(super::ENV, async |db| {
        new_user(QUERY_USER).insert(db).await?;
        // `execute_unprepared` would skip the trace: SeaORM only reports
        // statements built as a `Statement`.
        db.query_one_raw(Statement::from_string(
            db.get_database_backend(),
            "SELECT 1 FROM runique_test_no_such_table",
        ))
        .await?;
        Ok(())
    })
    .await;
    assert!(outcome.is_err(), "a failed query must fail the test");
    is_rolled_back(QUERY_USER).await
}

#[tokio::test]
async fn missing_env_file_is_reported() {
    let outcome = runique_test::<ADb>("runique_test_no_such_file.env", async |_| Ok(())).await;
    assert!(outcome.is_err(), "a missing env file must fail the test");
}
