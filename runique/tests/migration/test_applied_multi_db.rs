//! `makemigrations` reads what the database applied (`seaql_migrations`) the
//! same way on every engine: the check that the table exists is written per
//! engine in SeaORM, and answers "not supported" when the engine's feature is
//! missing from `sea-orm-migration`.
//!
//! Postgres/MySQL need `docker compose -f docker-compose.yml up -d` and their
//! URL in `.env.test`; without it, those tests return early.

use crate::helpers::{db, db_mysql, db_postgres};
use runique::sea_orm::{ConnectionTrait, DatabaseConnection};
use runique_cli::migration::cli_migration::applied_migrations;
use sea_query::{Alias, ColumnDef, Query, Table};
use serial_test::serial;

const OLDER: &str = "m000000_000001_runique_eihwaz_users";
const NEWER: &str = "m20260101_000000_create_notes_table";

/// Without the table nothing is applied; with it, its rows come back oldest
/// first, whatever the order they were written in.
async fn nothing_then_the_recorded_migrations(db: &DatabaseConnection) {
    let table = Alias::new("seaql_migrations");
    let (version, applied_at) = (Alias::new("version"), Alias::new("applied_at"));
    let drop = Table::drop().table(table.clone()).if_exists().to_owned();
    db.execute(&drop).await.expect("drop the table");

    let before = applied_migrations(db)
        .await
        .expect("read without the table");

    // The table as SeaORM creates it.
    db.execute(
        &Table::create()
            .table(table.clone())
            .col(
                ColumnDef::new(version.clone())
                    .string()
                    .not_null()
                    .primary_key(),
            )
            .col(ColumnDef::new(applied_at.clone()).big_integer().not_null())
            .to_owned(),
    )
    .await
    .expect("create the table");
    for name in [NEWER, OLDER] {
        db.execute(
            &Query::insert()
                .into_table(table.clone())
                .columns([version.clone(), applied_at.clone()])
                .values_panic([name.into(), 0i64.into()])
                .to_owned(),
        )
        .await
        .expect("record a migration");
    }

    let after = applied_migrations(db).await.expect("read the table");
    db.execute(&drop).await.expect("drop the table");

    assert!(before.is_empty(), "{before:?}");
    assert_eq!(after, [OLDER, NEWER]);
}

#[tokio::test]
async fn applied_migrations_are_read_on_sqlite() {
    nothing_then_the_recorded_migrations(&db::fresh_db().await).await;
}

#[tokio::test]
#[serial]
async fn applied_migrations_are_read_on_postgres() {
    let Some(db) = db_postgres::connect().await else {
        return;
    };
    nothing_then_the_recorded_migrations(&db).await;
}

#[tokio::test]
#[serial]
async fn applied_migrations_are_read_on_mysql() {
    let Some(db) = db_mysql::connect().await else {
        return;
    };
    nothing_then_the_recorded_migrations(&db).await;
}
