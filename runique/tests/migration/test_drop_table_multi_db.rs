//! What a migration that drops a table hands over to Runique, on the real
//! engines: the enum type of a dropped table is only dropped once no column
//! uses it any more.
//!
//! Only PostgreSQL has enum types. Elsewhere an enum column is plain text and
//! the same call does nothing.
//!
//! Postgres/MySQL need `docker compose -f docker-compose.yml up -d` and their
//! URL in `.env.test`; without it, those tests return early.

use crate::helpers::{db, db_mysql, db_postgres};
use runique::sea_orm::{ConnectionTrait, DatabaseConnection};
use runique_cli::migration::runtime::drop_enum_type;
use sea_orm_migration::SchemaManager;
use sea_query::extension::postgres::Type;
use sea_query::{Alias, ColumnDef, Expr, ExprTrait, Query, Table};
use serial_test::serial;

const TYPE: &str = "rq_drop_regime";
const FIRST: &str = "rq_drop_menus";
const SECOND: &str = "rq_drop_plats";

async fn type_exists(db: &DatabaseConnection) -> bool {
    db.query_one(
        &Query::select()
            .expr(Expr::val(1))
            .from((Alias::new("pg_catalog"), Alias::new("pg_type")))
            .and_where(Expr::col(Alias::new("typname")).eq(TYPE))
            .to_owned(),
    )
    .await
    .expect("look the type up")
    .is_some()
}

async fn drop_table(db: &DatabaseConnection, table: &str) {
    let drop = Table::drop()
        .table(Alias::new(table))
        .if_exists()
        .to_owned();
    db.execute(&drop).await.expect("drop a test table");
}

/// A table with one column of the enum type.
async fn table_with_the_type(db: &DatabaseConnection, table: &str) {
    db.execute(
        &Table::create()
            .table(Alias::new(table))
            .col(
                ColumnDef::new(Alias::new("id"))
                    .integer()
                    .not_null()
                    .primary_key(),
            )
            .col(
                ColumnDef::new(Alias::new("regime"))
                    .enumeration(Alias::new(TYPE), [Alias::new("a"), Alias::new("b")])
                    .not_null(),
            )
            .to_owned(),
    )
    .await
    .expect("create a test table");
}

// Two tables share the type. Dropping the first leaves it; dropping the
// second lets it go; asking again for a type already gone is not an error.
// The name is given in upper case, as a model may write it.
#[tokio::test]
#[serial]
async fn an_enum_type_goes_with_its_last_column_on_postgres() {
    let Some(db) = db_postgres::connect().await else {
        return;
    };
    for table in [FIRST, SECOND] {
        drop_table(&db, table).await;
    }
    db.execute(&Type::drop().if_exists().name(Alias::new(TYPE)).to_owned())
        .await
        .expect("drop the type");
    db.execute(
        &Type::create()
            .as_enum(Alias::new(TYPE))
            .values([Alias::new("a"), Alias::new("b")])
            .to_owned(),
    )
    .await
    .expect("create the type");
    table_with_the_type(&db, FIRST).await;
    table_with_the_type(&db, SECOND).await;
    let manager = SchemaManager::new(&db);
    let asked = TYPE.to_uppercase();

    drop_enum_type(&manager, &asked)
        .await
        .expect("both tables use it");
    let with_two = type_exists(&db).await;

    drop_table(&db, FIRST).await;
    drop_enum_type(&manager, &asked)
        .await
        .expect("one table uses it");
    let with_one = type_exists(&db).await;

    drop_table(&db, SECOND).await;
    drop_enum_type(&manager, &asked)
        .await
        .expect("nothing uses it");
    let with_none = type_exists(&db).await;
    let again = drop_enum_type(&manager, &asked).await;

    assert!(with_two, "a type two tables use must stay");
    assert!(with_one, "a type one table still uses must stay");
    assert!(!with_none, "a type nothing uses must go");
    assert!(again.is_ok(), "{again:?}");
}

// No enum type on MySQL nor on SQLite: the call does nothing and succeeds.
#[tokio::test]
#[serial]
async fn dropping_an_enum_type_is_nothing_on_mysql() {
    let Some(db) = db_mysql::connect().await else {
        return;
    };
    drop_enum_type(&SchemaManager::new(&db), TYPE)
        .await
        .expect("nothing to drop");
}

#[tokio::test]
async fn dropping_an_enum_type_is_nothing_on_sqlite() {
    let db = db::fresh_db().await;
    drop_enum_type(&SchemaManager::new(&db), TYPE)
        .await
        .expect("nothing to drop");
}
