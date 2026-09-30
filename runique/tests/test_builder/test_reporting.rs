//! The verdict tells the truth: a swallowed error or a transaction ended
//! from inside fails the test, an expected refusal doesn't, and a test with
//! no database named never runs at all.
use super::helpers::{Scratch, count, insert};
use runique::db::ADb;
use runique::runique_test::{expect_db_error, runique_test};
use runique::sea_orm::ConnectionTrait;
use std::sync::atomic::{AtomicBool, Ordering};

#[tokio::test]
async fn a_swallowed_query_error_fails_the_test() {
    let scratch = Scratch::new("swallowed");
    scratch.outside().await;

    let outcome = runique_test::<ADb>(&scratch.sqlite_env(&[]), async |db| {
        insert(db, "a").await?;
        // Business code that turns a failed query into "nothing happened".
        insert(db, "a").await.ok();
        Ok(())
    })
    .await;

    assert!(
        outcome.is_err(),
        "the unique violation was swallowed, the test must fail"
    );
}

#[tokio::test]
async fn a_swallowed_raw_statement_error_fails_the_test_too() {
    let scratch = Scratch::new("swallowed_raw");
    scratch.outside().await;

    // `execute_unprepared` isn't reported by SeaORM: `ADb` has to trace it.
    let outcome = runique_test::<ADb>(&scratch.sqlite_env(&[]), async |db| {
        db.execute_unprepared("INSERT INTO no_such_table VALUES (1)")
            .await
            .ok();
        Ok(())
    })
    .await;

    assert!(outcome.is_err());
}

#[tokio::test]
async fn an_expected_refusal_does_not_fail_the_test() {
    let scratch = Scratch::new("expected");
    let outside = scratch.outside().await;

    let outcome = runique_test::<ADb>(&scratch.sqlite_env(&[]), async |db| {
        insert(db, "a").await?;
        expect_db_error(db, async |sp| insert(sp, "a").await).await?;
        // The savepoint kept the transaction usable.
        insert(db, "b").await?;
        assert_eq!(count(db).await, 2);
        Ok(())
    })
    .await;

    assert!(outcome.is_ok(), "{outcome:?}");
    assert_eq!(count(&outside).await, 0);
}

#[tokio::test]
async fn expect_db_error_says_so_when_the_database_accepts() {
    let scratch = Scratch::new("not_refused");
    scratch.outside().await;

    let outcome = runique_test::<ADb>(&scratch.sqlite_env(&[]), async |db| {
        let accepted = expect_db_error(db, async |sp| insert(sp, "fine").await).await;
        assert!(accepted.is_err(), "an accepted insert isn't a refusal");
        assert_eq!(
            count(db).await,
            0,
            "the savepoint undid the accepted insert"
        );
        Ok(())
    })
    .await;

    assert!(outcome.is_ok(), "{outcome:?}");
}

#[tokio::test]
async fn a_commit_inside_the_test_fails_it_on_sqlite() {
    let scratch = Scratch::new("commit_sqlite");
    let outside = scratch.outside().await;

    let outcome = runique_test::<ADb>(&scratch.sqlite_env(&[]), async |db| {
        insert(db, "leaked").await?;
        db.execute_unprepared("COMMIT").await?;
        Ok(())
    })
    .await;

    assert!(
        outcome.is_err(),
        "the test must not pass once its writes were committed"
    );
    // The damage is done; the point is that it's reported.
    assert_eq!(count(&outside).await, 1);
}

#[tokio::test]
async fn an_env_file_without_a_database_never_runs_the_handler() {
    let scratch = Scratch::new("no_db");
    let env = scratch.env_file(&["DEBUG=true".to_string()]);
    let ran = AtomicBool::new(false);

    let outcome = runique_test::<ADb>(&env, async |_| {
        ran.store(true, Ordering::SeqCst);
        Ok(())
    })
    .await;

    assert!(outcome.is_err());
    assert!(!ran.load(Ordering::SeqCst));
}

#[tokio::test]
async fn a_bad_timeout_value_never_runs_the_handler() {
    let scratch = Scratch::new("bad_timeout");
    let env = scratch.sqlite_env(&["RUNIQUE_TEST_TIMEOUT=soon"]);
    let ran = AtomicBool::new(false);

    let outcome = runique_test::<ADb>(&env, async |_| {
        ran.store(true, Ordering::SeqCst);
        Ok(())
    })
    .await;

    assert!(outcome.is_err());
    assert!(!ran.load(Ordering::SeqCst));
}

#[tokio::test]
async fn a_failure_message_comes_back_with_the_error() {
    let scratch = Scratch::new("message");
    scratch.outside().await;

    let failure = runique_test::<ADb>(&scratch.sqlite_env(&[]), async |_| {
        Err(runique::sea_orm::DbErr::Custom("marker 7f3a".into()))
    })
    .await
    .expect_err("the handler failed");

    assert!(
        failure.message.contains("marker 7f3a"),
        "{}",
        failure.message
    );
}
