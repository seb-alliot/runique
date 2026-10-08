//! `eihwaz_users` as the framework creates it — the database itself refuses an
//! active account that was never activated (`is_active` without
//! `activated_at`), whatever writes it: checked here in raw SQL, without the
//! ORM, on every engine available (SQLite always, Postgres and MariaDB when
//! their container is).

use crate::helpers::pk::{pk_sql_literal, pk_sql_literal_pg};
use crate::helpers::{db, db_mariadb, db_postgres};
use runique::admin::table_admin::migrations_table::{
    ACTIVE_NEEDS_ACTIVATION, EihwazUsersMigration,
};
use runique::sea_orm::{ConnectionTrait, DatabaseConnection, TransactionTrait};
use sea_orm_migration::{MigrationTrait, SchemaManager};
use serial_test::serial;

async fn sql(db: &DatabaseConnection, statement: &str) -> Result<u64, String> {
    db.execute_unprepared(statement)
        .await
        .map(|r| r.rows_affected())
        .map_err(|e| e.to_string())
}

/// `id(n)`: the primary key literal for this engine.
async fn check_guarantee(db: &DatabaseConnection, id: impl Fn(u32) -> String) {
    let manager = SchemaManager::new(db);
    EihwazUsersMigration
        .up(&manager)
        .await
        .expect("users table");

    let insert = |n: u32, name: &str, active: bool| {
        format!(
            "INSERT INTO eihwaz_users (id, username, email, password, is_active, is_staff, is_superuser) \
             VALUES ({}, '{name}', '{name}@example.com', 'h', {active}, FALSE, FALSE)",
            id(n)
        )
    };

    let refused = |r: Result<u64, String>| {
        let e = r.expect_err("refused by the database");
        assert!(
            e.contains(ACTIVE_NEEDS_ACTIVATION),
            "refused for the right reason: {e}"
        );
    };

    // Active and never activated: refused on insert and on update.
    refused(sql(db, &insert(3, "fake_active", true)).await);
    sql(db, &insert(4, "pending", false)).await.unwrap();
    refused(
        sql(
            db,
            "UPDATE eihwaz_users SET is_active = TRUE WHERE username = 'pending'",
        )
        .await,
    );

    // Allowed: activation by its owner, block and unblock by the staff.
    sql(
        db,
        "UPDATE eihwaz_users SET is_active = TRUE, activated_at = CURRENT_TIMESTAMP \
         WHERE username = 'pending'",
    )
    .await
    .unwrap();
    sql(
        db,
        "UPDATE eihwaz_users SET is_active = FALSE WHERE username = 'pending'",
    )
    .await
    .unwrap();
    sql(
        db,
        "UPDATE eihwaz_users SET is_active = TRUE WHERE username = 'pending'",
    )
    .await
    .unwrap();
}

#[tokio::test]
#[serial]
async fn sqlite_refuses_an_active_account_never_activated() {
    let conn = db::fresh_db().await;
    check_guarantee(&conn, pk_sql_literal).await;
}

#[tokio::test]
#[serial]
async fn postgres_refuses_an_active_account_never_activated() {
    let Some(conn) = db_postgres::connect().await else {
        return;
    };
    sql(&conn, "DROP TABLE IF EXISTS eihwaz_users CASCADE")
        .await
        .unwrap();
    check_guarantee(&conn, pk_sql_literal_pg).await;
    sql(&conn, "DROP TABLE IF EXISTS eihwaz_users CASCADE")
        .await
        .unwrap();
}

/// Drops `eihwaz_users` despite the tables of other tests referencing it on the
/// shared MariaDB. `FOREIGN_KEY_CHECKS` is a per-connection variable and the
/// connection is a pool: the transaction pins `SET`/`DROP`/`SET` to a single
/// connection (the `DROP` commits it implicitly, the connection stays the same).
async fn drop_users_mariadb(db: &DatabaseConnection) {
    let txn = db.begin().await.unwrap();
    for sql in [
        "SET FOREIGN_KEY_CHECKS = 0",
        "DROP TABLE IF EXISTS eihwaz_users",
        "SET FOREIGN_KEY_CHECKS = 1",
    ] {
        txn.execute_unprepared(sql).await.unwrap();
    }
    txn.commit().await.unwrap();
}

#[tokio::test]
#[serial]
async fn mariadb_refuses_an_active_account_never_activated() {
    let Some(conn) = db_mariadb::connect().await else {
        return;
    };
    drop_users_mariadb(&conn).await;
    // A UUID is stored as BINARY(16) on MariaDB: the hex literal, as on SQLite.
    check_guarantee(&conn, pk_sql_literal).await;
    drop_users_mariadb(&conn).await;
}
