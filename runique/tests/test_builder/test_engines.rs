//! Engine-specific ways of ending the transaction behind the builder's back.
//! On Postgres and MySQL the final `ROLLBACK` then succeeds without undoing
//! anything, so only the builder's own check can catch it.
//!
//! Need `docker compose up -d` and `DATABASE_URL_PG` / `DATABASE_URL_MYSQL`
//! in `.env.test`; without them each test returns right away.
use super::helpers::{ITEMS, Scratch, count, create_items, insert};
#[cfg(feature = "mysql")]
use crate::helpers::db_mysql;
#[cfg(feature = "postgres")]
use crate::helpers::db_postgres;
use runique::db::ADb;
use runique::runique_test::runique_test;
use runique::sea_orm::{ConnectionTrait, DatabaseConnection};
use serial_test::serial;

/// An env file pointing to this copy's own database for the engine `url_key`
/// names (see `helpers::db_isolation`).
async fn env_for(scratch: &Scratch, url_key: &str) -> String {
    let url = crate::helpers::db_isolation::isolated_url(url_key)
        .await
        .expect("set by the Docker helper");
    scratch.env_file(&[format!("DATABASE_URL={url}")])
}

async fn fresh_items(conn: &DatabaseConnection) {
    conn.execute_unprepared(&format!("DROP TABLE IF EXISTS {ITEMS}"))
        .await
        .expect("drop items");
    create_items(conn).await;
}

#[tokio::test]
#[serial]
#[cfg(feature = "postgres")]
async fn postgres_rolls_back_normally() {
    let Some(outside) = db_postgres::connect().await else {
        return;
    };
    fresh_items(&outside).await;
    let scratch = Scratch::new("pg_rollback");

    let outcome = runique_test::<ADb>(&env_for(&scratch, "DATABASE_URL_PG").await, async |db| {
        insert(db, "a").await?;
        Ok(())
    })
    .await;

    assert!(outcome.is_ok(), "{outcome:?}");
    assert_eq!(count(&outside).await, 0);
    db_postgres::exec(&outside, &format!("DROP TABLE {ITEMS}")).await;
}

#[tokio::test]
#[serial]
#[cfg(feature = "postgres")]
async fn postgres_commit_inside_the_test_is_caught() {
    let Some(outside) = db_postgres::connect().await else {
        return;
    };
    fresh_items(&outside).await;
    let scratch = Scratch::new("pg_commit");

    let outcome = runique_test::<ADb>(&env_for(&scratch, "DATABASE_URL_PG").await, async |db| {
        insert(db, "leaked").await?;
        db.execute_unprepared("COMMIT").await?;
        Ok(())
    })
    .await;

    // Postgres answers the final ROLLBACK with a mere warning: without the
    // builder's own check this test would be green.
    assert!(
        outcome.is_err(),
        "a committed test transaction must fail the test"
    );
    assert_eq!(count(&outside).await, 1);
    db_postgres::exec(&outside, &format!("DROP TABLE {ITEMS}")).await;
}

#[tokio::test]
#[serial]
#[cfg(feature = "mysql")]
async fn mysql_ddl_inside_the_test_is_caught() {
    let Some(outside) = db_mysql::connect().await else {
        return;
    };
    fresh_items(&outside).await;
    let scratch = Scratch::new("mysql_ddl");

    let outcome = runique_test::<ADb>(&env_for(&scratch, "DATABASE_URL_MYSQL").await, async |db| {
        insert(db, "leaked").await?;
        // Commits implicitly on MySQL, taking the insert above with it.
        db.execute_unprepared("CREATE TABLE rq_builder_other (x INT)")
            .await?;
        Ok(())
    })
    .await;

    assert!(
        outcome.is_err(),
        "DDL ended the transaction, the test must fail"
    );
    assert_eq!(count(&outside).await, 1);
    db_mysql::exec(&outside, "DROP TABLE IF EXISTS rq_builder_other").await;
    db_mysql::exec(&outside, &format!("DROP TABLE {ITEMS}")).await;
}

#[tokio::test]
#[serial]
#[cfg(feature = "mysql")]
async fn mysql_rolls_back_normally() {
    let Some(outside) = db_mysql::connect().await else {
        return;
    };
    fresh_items(&outside).await;
    let scratch = Scratch::new("mysql_rollback");

    let outcome = runique_test::<ADb>(&env_for(&scratch, "DATABASE_URL_MYSQL").await, async |db| {
        insert(db, "a").await?;
        Ok(())
    })
    .await;

    assert!(outcome.is_ok(), "{outcome:?}");
    assert_eq!(count(&outside).await, 0);
    db_mysql::exec(&outside, &format!("DROP TABLE {ITEMS}")).await;
}
