//! `eihwaz_groupes_droits` as the framework creates it: the database itself
//! refuses a row with no holder or two, a group's row on a framework table, a
//! column row carrying a table right, and a duplicate, whatever writes it.
//! Raw SQL, without the ORM, on every engine available.

use crate::helpers::pk::{pk_sql_literal, pk_sql_literal_pg};
use crate::helpers::{db, db_mariadb, db_postgres};
use runique::migrations_table::{
    AdminTableMigration, DROITS_COLUMN_ROW_FLAGS, DROITS_GROUP_NEVER_FRAMEWORK, DROITS_ONE_HOLDER,
    EihwazUsersMigration,
};
use runique::sea_orm::sea_query::{Alias, Expr, Query};
use runique::sea_orm::{ConnectionTrait, DatabaseConnection, ExprTrait, TransactionTrait};
use runique::utils::constante::resource_key::FRAMEWORK_KEYS;
use sea_orm_migration::{MigrationTrait, SchemaManager};
use serial_test::serial;

/// Children first, so a plain DROP never trips over a FK.
const TABLES: &[&str] = &[
    "eihwaz_sessions",
    "eihwaz_users_groupes",
    "eihwaz_groupes_droits",
    "eihwaz_groupes",
    "eihwaz_users",
];

async fn sql(db: &DatabaseConnection, statement: &str) -> Result<u64, String> {
    db.execute_unprepared(statement)
        .await
        .map(|r| r.rows_affected())
        .map_err(|e| e.to_string())
}

fn refused(r: Result<u64, String>, check: &str) {
    let e = r.expect_err("refused by the database");
    assert!(e.contains(check), "refused by {check}: {e}");
}

/// Postgres and MariaDB name the index; SQLite only lists its columns.
fn duplicate(r: Result<u64, String>, index: &str, holder: &str) {
    let e = r.expect_err("a duplicate");
    let sqlite = format!("UNIQUE constraint failed: eihwaz_groupes_droits.{holder}");
    assert!(
        e.contains(index) || e.contains(&sqlite),
        "duplicate on {index}: {e}"
    );
}

/// `id(n)`: the account primary key literal for this engine.
async fn check_rights_table(db: &DatabaseConnection, id: impl Fn(u32) -> String) {
    let manager = SchemaManager::new(db);
    EihwazUsersMigration
        .up(&manager)
        .await
        .expect("users table");
    AdminTableMigration
        .up(&manager)
        .await
        .expect("admin tables");

    for n in [1, 2, 3] {
        sql(
            db,
            &format!(
                "INSERT INTO eihwaz_users (id, username, email, password) \
                 VALUES ({}, 'u{n}', 'u{n}@example.com', 'h')",
                id(n)
            ),
        )
        .await
        .unwrap();
    }
    sql(
        db,
        &format!(
            "INSERT INTO eihwaz_groupes (id, nom, responsable_id) VALUES (1, 'support', {})",
            id(1)
        ),
    )
    .await
    .unwrap();

    let (u1, u2) = (id(1), id(2));
    let insert = |groupe: &str, user: &str, key: &str, colonne: &str, flag: &str| {
        format!(
            "INSERT INTO eihwaz_groupes_droits (groupe_id, user_id, resource_key, colonne, {flag}) \
             VALUES ({groupe}, {user}, '{key}', '{colonne}', TRUE)"
        )
    };

    // Exactly one holder: a group or an account, never none, never both.
    sql(db, &insert("1", "NULL", "articles", "", "can_read"))
        .await
        .unwrap();
    sql(db, &insert("NULL", &u1, "articles", "", "can_read"))
        .await
        .unwrap();
    refused(
        sql(db, &insert("NULL", "NULL", "articles", "x", "can_read")).await,
        DROITS_ONE_HOLDER,
    );
    refused(
        sql(db, &insert("1", &u2, "articles", "y", "can_read")).await,
        DROITS_ONE_HOLDER,
    );

    // A group never on a framework table; an admin account may be.
    for key in FRAMEWORK_KEYS {
        refused(
            sql(db, &insert("1", "NULL", key, "", "can_read")).await,
            DROITS_GROUP_NEVER_FRAMEWORK,
        );
        sql(db, &insert("NULL", &u1, key, "", "can_read"))
            .await
            .unwrap();
    }

    // A column row reads, creates, updates; delete and `_own` are table rights.
    for flag in ["can_read", "can_create", "can_update"] {
        sql(db, &insert("NULL", &u1, "articles", flag, flag))
            .await
            .unwrap();
    }
    for flag in [
        "can_read_own",
        "can_update_own",
        "can_delete",
        "can_delete_own",
    ] {
        refused(
            sql(db, &insert("NULL", &u1, "articles", "titre", flag)).await,
            DROITS_COLUMN_ROW_FLAGS,
        );
        sql(db, &insert("1", "NULL", &format!("t_{flag}"), "", flag))
            .await
            .unwrap();
    }

    // One row per (holder, table, column). The two accounts' rows on
    // ('articles', '') both have a NULL groupe_id: they don't collide.
    duplicate(
        sql(db, &insert("1", "NULL", "articles", "", "can_create")).await,
        "uq_eihwaz_groupes_droits_groupe",
        "groupe_id",
    );
    duplicate(
        sql(db, &insert("NULL", &u1, "articles", "", "can_update")).await,
        "uq_eihwaz_groupes_droits_user",
        "user_id",
    );
    sql(db, &insert("1", "NULL", "articles", "titre", "can_read"))
        .await
        .unwrap();
    sql(db, &insert("NULL", &u2, "articles", "", "can_read"))
        .await
        .unwrap();

    // Deleting the holder deletes its rows: nothing left to delete after it,
    // while the other account's rows are still there.
    sql(db, "DELETE FROM eihwaz_groupes WHERE id = 1")
        .await
        .unwrap();
    let left = sql(db, "DELETE FROM eihwaz_groupes_droits WHERE groupe_id = 1").await;
    assert_eq!(left, Ok(0), "the group's rights went with it");
    sql(db, &format!("DELETE FROM eihwaz_users WHERE id = {u1}"))
        .await
        .unwrap();
    let left = sql(
        db,
        &format!("DELETE FROM eihwaz_groupes_droits WHERE user_id = {u1}"),
    )
    .await;
    assert_eq!(left, Ok(0), "the account's rights went with it");
    let kept = sql(
        db,
        &format!("DELETE FROM eihwaz_groupes_droits WHERE user_id = {u2}"),
    )
    .await;
    assert_eq!(kept, Ok(1), "another account keeps its rights");

    // A group always has a real responsable: none is refused, and his account
    // can't be deleted while he answers for one. `repris_par` only marks a
    // group to hand over: its account goes, the mark is cleared.
    let u3 = id(3);
    assert!(
        sql(
            db,
            "INSERT INTO eihwaz_groupes (id, nom) VALUES (2, 'sans')"
        )
        .await
        .is_err(),
        "a group without a responsable"
    );
    sql(
        db,
        &format!(
            "INSERT INTO eihwaz_groupes (id, nom, responsable_id, repris_par) \
             VALUES (2, 'compta', {u2}, {u3})"
        ),
    )
    .await
    .unwrap();
    assert!(
        sql(db, &format!("DELETE FROM eihwaz_users WHERE id = {u2}"))
            .await
            .is_err(),
        "the responsable's account stays while he answers for a group"
    );
    sql(db, &format!("DELETE FROM eihwaz_users WHERE id = {u3}"))
        .await
        .unwrap();
    let cleared = sql(
        db,
        "UPDATE eihwaz_groupes SET nom = 'compta' WHERE id = 2 AND repris_par IS NULL",
    )
    .await;
    assert_eq!(cleared, Ok(1), "repris_par cleared, the group kept");
}

#[tokio::test]
#[serial]
async fn sqlite_rights_table_guarantees() {
    let conn = db::fresh_db().await;
    check_rights_table(&conn, pk_sql_literal).await;
}

async fn drop_postgres(db: &DatabaseConnection) {
    for table in TABLES {
        sql(db, &format!("DROP TABLE IF EXISTS {table} CASCADE"))
            .await
            .unwrap();
    }
}

#[tokio::test]
#[serial]
async fn postgres_rights_table_guarantees() {
    let Some(conn) = db_postgres::connect().await else {
        return;
    };
    drop_postgres(&conn).await;
    check_rights_table(&conn, pk_sql_literal_pg).await;
    drop_postgres(&conn).await;
}

/// Same pinning as `drop_users_mariadb`: `FOREIGN_KEY_CHECKS` is per
/// connection, the transaction keeps `SET`/`DROP`/`SET` on one.
///
/// Other tests leave tables on this database (`contributions` from the demo
/// migrations, `eihwaz_reset_tokens`, …) with a foreign key onto these.
/// MariaDB keeps that key when the parent is dropped with checks off, then
/// refuses to recreate the parent if the key's type differs — errno 150.
/// Under `pk-uuid` the new `id` is `binary(16)` and a leftover key is `int`.
async fn drop_mariadb(db: &DatabaseConnection) {
    let txn = db.begin().await.unwrap();
    txn.execute_unprepared("SET FOREIGN_KEY_CHECKS = 0")
        .await
        .unwrap();
    let referencing = Query::select()
        .distinct()
        .column(Alias::new("TABLE_NAME"))
        .from((
            Alias::new("information_schema"),
            Alias::new("KEY_COLUMN_USAGE"),
        ))
        .and_where(Expr::col(Alias::new("TABLE_SCHEMA")).eq(Expr::cust("DATABASE()")))
        .and_where(Expr::col(Alias::new("REFERENCED_TABLE_NAME")).is_in([
            "eihwaz_users",
            "eihwaz_groupes",
            "eihwaz_groupes_droits",
            "eihwaz_users_groupes",
            "eihwaz_sessions",
        ]))
        .to_owned();
    let referencing = txn
        .query_all(&referencing)
        .await
        .expect("tables that reference the framework tables");
    for row in referencing {
        let name: String = row.try_get("", "TABLE_NAME").expect("table name");
        let ident = name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_')
            .then_some(name.as_str())
            .expect("table name");
        txn.execute_unprepared(&format!("DROP TABLE IF EXISTS `{ident}`"))
            .await
            .unwrap();
    }
    for table in TABLES {
        txn.execute_unprepared(&format!("DROP TABLE IF EXISTS `{table}`"))
            .await
            .unwrap();
    }
    txn.execute_unprepared("SET FOREIGN_KEY_CHECKS = 1")
        .await
        .unwrap();
    txn.commit().await.unwrap();
}

#[tokio::test]
#[serial]
async fn mariadb_rights_table_guarantees() {
    let Some(conn) = db_mariadb::connect().await else {
        return;
    };
    drop_mariadb(&conn).await;
    check_rights_table(&conn, pk_sql_literal).await;
    drop_mariadb(&conn).await;
}
