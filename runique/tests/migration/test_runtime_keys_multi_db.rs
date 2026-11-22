//! A foreign key added to or removed from an existing table, through the
//! functions a generated migration calls (`migration::runtime`).
//!
//! PostgreSQL and MySQL do it in place. They refuse to drop a table as long
//! as a key points to it, even when both tables are empty: removing the key
//! first is what makes the drop possible. SQLite can't change the keys of an
//! existing table: the migration is refused before it changes anything.
//!
//! The tables carry Runique's primary key type, so the suites run under
//! `big-pk` and `pk-uuid` exercise a key to a `bigint` and to a `uuid`.
//!
//! Postgres/MySQL need `docker compose -f docker-compose.yml up -d` and their
//! URL in `.env.test`; without it, those tests return early.

use crate::helpers::{db, db_mysql, db_postgres};
use runique::migration::runtime::{add_foreign_key, drop_foreign_key, key_changes_supported};
use runique::sea_orm::{ConnectionTrait, DatabaseConnection};
use sea_orm_migration::SchemaManager;
use sea_query::{Alias, ColumnDef, ForeignKey, ForeignKeyCreateStatement, Table};
use serial_test::serial;

const PARENT: &str = "rq_keys_parent";
const CHILD: &str = "rq_keys_child";
const KEY: &str = "rq_keys_child_parent_id_rq_keys_parent_fkey";

fn key() -> ForeignKeyCreateStatement {
    ForeignKey::create()
        .name(KEY)
        .from(Alias::new(CHILD), Alias::new("parent_id"))
        .to(Alias::new(PARENT), Alias::new("id"))
        .to_owned()
}

async fn drop_both(db: &DatabaseConnection) {
    for table in [CHILD, PARENT] {
        let drop = Table::drop().table(Alias::new(table)).if_exists().to_owned();
        db.execute(&drop).await.expect("drop a test table");
    }
}

/// A column of the type of Runique's primary key: `i32` by default, `i64`
/// with `big-pk`, `Uuid` with `pk-uuid`. The key under test then points to
/// what a model's key points to under each variant.
fn pk_typed(name: &str) -> ColumnDef {
    let mut column = ColumnDef::new(Alias::new(name));
    #[cfg(feature = "pk-uuid")]
    column.uuid();
    #[cfg(all(feature = "big-pk", not(feature = "pk-uuid")))]
    column.big_integer();
    #[cfg(not(any(feature = "big-pk", feature = "pk-uuid")))]
    column.integer();
    column
}

/// Two empty tables, no key between them yet.
async fn two_tables(db: &DatabaseConnection) {
    drop_both(db).await;
    for (table, extra) in [(PARENT, None), (CHILD, Some("parent_id"))] {
        let mut create = Table::create();
        create
            .table(Alias::new(table))
            .col(pk_typed("id").not_null().primary_key());
        if let Some(column) = extra {
            create.col(pk_typed(column));
        }
        db.execute(&create.to_owned()).await.expect("create a test table");
    }
}

async fn drop_parent(db: &DatabaseConnection) -> Result<(), runique::sea_orm::DbErr> {
    db.execute(&Table::drop().table(Alias::new(PARENT)).to_owned())
        .await
        .map(|_| ())
}

/// The key is added, holds the parent in place, is removed, and the parent
/// can then go.
async fn a_key_is_added_then_removed(db: &DatabaseConnection) {
    two_tables(db).await;
    let manager = SchemaManager::new(db);

    key_changes_supported(&manager, CHILD).expect("a server changes keys in place");
    add_foreign_key(&manager, CHILD, key()).await.expect("add the key");
    let held = drop_parent(db).await;

    drop_foreign_key(&manager, CHILD, KEY).await.expect("remove the key");
    let freed = drop_parent(db).await;
    drop_both(db).await;

    assert!(held.is_err(), "the key must hold the parent, even empty");
    assert!(freed.is_ok(), "{freed:?}");
}

#[tokio::test]
#[serial]
async fn a_key_is_added_then_removed_on_postgres() {
    let Some(db) = db_postgres::connect().await else {
        return;
    };
    a_key_is_added_then_removed(&db).await;
}

#[tokio::test]
#[serial]
async fn a_key_is_added_then_removed_on_mysql() {
    let Some(db) = db_mysql::connect().await else {
        return;
    };
    a_key_is_added_then_removed(&db).await;
}

// SQLite: refused at each entry point, and the tables are as they were.
#[tokio::test]
async fn a_key_change_is_refused_on_sqlite() {
    let db = db::fresh_db().await;
    two_tables(&db).await;
    let manager = SchemaManager::new(&db);

    assert!(key_changes_supported(&manager, CHILD).is_err());
    assert!(add_foreign_key(&manager, CHILD, key()).await.is_err());
    assert!(drop_foreign_key(&manager, CHILD, KEY).await.is_err());

    assert!(manager.has_table(PARENT).await.unwrap());
    assert!(manager.has_table(CHILD).await.unwrap());
    drop_both(&db).await;
}
