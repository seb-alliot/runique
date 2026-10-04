//! The admin's SQL written with sea-query instead of raw strings — checked on
//! every engine available (SQLite always, Postgres and MariaDB when their
//! container is): the many-to-many links of generated resources, and the FK
//! labels / FK search (`CAST(id AS TEXT)` was invalid on MySQL/MariaDB).

use crate::helpers::{db, db_mariadb, db_postgres};
use runique::admin::helper::fk_resolve::{fetch_fk_label_map, fetch_fk_matching_ids};
use runique::admin::helper::m2m::write_links;
use runique::sea_orm::sea_query::Value;
use runique::sea_orm::{ConnectionTrait, DatabaseConnection, Statement, TransactionTrait};
use runique::utils::aliases::StrMap;
use serial_test::serial;

async fn sql(db: &DatabaseConnection, statement: &str) {
    db.execute_unprepared(statement).await.expect(statement);
}

async fn links(db: &DatabaseConnection, table: &str) -> Vec<(i32, i32)> {
    db.query_all_raw(Statement::from_string(
        db.get_database_backend(),
        format!("SELECT owner_id, tag_id FROM {table} ORDER BY owner_id, tag_id"),
    ))
    .await
    .unwrap()
    .iter()
    .map(|r| {
        (
            r.try_get("", "owner_id").unwrap(),
            r.try_get("", "tag_id").unwrap(),
        )
    })
    .collect()
}

fn form(keys: &[&str]) -> StrMap {
    keys.iter()
        .map(|k| (k.to_string(), "on".to_string()))
        .collect()
}

async fn check_m2m(db: &DatabaseConnection) {
    sql(db, "DROP TABLE IF EXISTS native_links").await;
    sql(db, "DROP TABLE IF EXISTS native_strict_links").await;
    sql(
        db,
        "CREATE TABLE native_links (owner_id INTEGER NOT NULL, tag_id INTEGER NOT NULL, \
         PRIMARY KEY (owner_id, tag_id))",
    )
    .await;
    sql(
        db,
        "INSERT INTO native_links (owner_id, tag_id) VALUES (2, 3)",
    )
    .await;

    let owner = || Value::from(1i32);
    let prefix = "m2m_tags__";
    write_links(
        db,
        "native_links",
        "owner_id",
        "tag_id",
        owner(),
        &form(&["m2m_tags__3", "m2m_tags__5", "title", "m2m_tags__not an id"]),
        prefix,
        false,
    )
    .await
    .expect("links written");
    assert_eq!(links(db, "native_links").await, [(1, 3), (1, 5), (2, 3)]);

    // An edit replaces this owner's links, and only theirs.
    write_links(
        db,
        "native_links",
        "owner_id",
        "tag_id",
        owner(),
        &form(&["m2m_tags__5", "m2m_tags__7"]),
        prefix,
        true,
    )
    .await
    .expect("links replaced");
    assert_eq!(links(db, "native_links").await, [(1, 5), (1, 7), (2, 3)]);

    // A failure inside the transaction leaves the links as they were: the
    // delete of the old ones is rolled back with the failed insert.
    sql(
        db,
        "CREATE TABLE native_strict_links (owner_id INTEGER NOT NULL, tag_id INTEGER NOT NULL, \
         note VARCHAR(10) NOT NULL)",
    )
    .await;
    sql(
        db,
        "INSERT INTO native_strict_links (owner_id, tag_id, note) VALUES (1, 9, 'kept')",
    )
    .await;
    let txn = db.begin().await.unwrap();
    let failed = write_links(
        &txn,
        "native_strict_links",
        "owner_id",
        "tag_id",
        owner(),
        &form(&["m2m_tags__4"]),
        prefix,
        true,
    )
    .await;
    assert!(failed.is_err(), "the insert misses a NOT NULL column");
    drop(txn);
    assert_eq!(
        links(db, "native_strict_links").await,
        [(1, 9)],
        "rolled back"
    );

    sql(db, "DROP TABLE native_links").await;
    sql(db, "DROP TABLE native_strict_links").await;
}

async fn check_fk(db: &DatabaseConnection) {
    sql(db, "DROP TABLE IF EXISTS native_fk_labels").await;
    sql(
        db,
        "CREATE TABLE native_fk_labels (id INTEGER NOT NULL PRIMARY KEY, name VARCHAR(50) NOT NULL)",
    )
    .await;
    sql(
        db,
        "INSERT INTO native_fk_labels (id, name) VALUES (1, 'Alpha'), (2, 'Beta')",
    )
    .await;

    let labels = fetch_fk_label_map(
        db,
        "native_fk_labels",
        "name",
        &["1".to_string(), "2".to_string(), "3".to_string()],
    )
    .await;
    assert_eq!(labels.get("1").map(String::as_str), Some("Alpha"));
    assert_eq!(labels.get("2").map(String::as_str), Some("Beta"));
    assert_eq!(labels.len(), 2);

    assert_eq!(
        fetch_fk_matching_ids(db, "native_fk_labels", "name", "ALP").await,
        ["1"]
    );
    sql(db, "DROP TABLE native_fk_labels").await;
}

#[tokio::test]
#[serial]
async fn sqlite_native_admin_sql() {
    let conn = db::fresh_db().await;
    check_m2m(&conn).await;
    check_fk(&conn).await;
}

#[tokio::test]
#[serial]
async fn postgres_native_admin_sql() {
    let Some(conn) = db_postgres::connect().await else {
        return;
    };
    check_m2m(&conn).await;
    check_fk(&conn).await;
}

#[tokio::test]
#[serial]
async fn mariadb_native_admin_sql() {
    let Some(conn) = db_mariadb::connect().await else {
        return;
    };
    check_m2m(&conn).await;
    check_fk(&conn).await;
}
