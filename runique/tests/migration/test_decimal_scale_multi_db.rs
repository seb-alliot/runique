//! A `decimal` field keeps its cents on every engine. `makemigrations` writes
//! it as `.decimal_len(max_digits, decimal_places)` (see `tests_pipeline.rs`):
//! a bare `.decimal()` is what MySQL/MariaDB read as `DECIMAL(10, 0)`, where
//! 9.99 came back as 10, silently.
//!
//! Postgres/MariaDB need `docker compose -f docker-compose.yml up -d` and their
//! URL in `.env.test`; without it, those tests return early.

use crate::helpers::{db, db_mariadb, db_postgres};
use runique::sea_orm::{ConnectionTrait, DatabaseConnection, prelude::Decimal};
use sea_query::{Alias, ColumnDef, Expr, Query, Table};
use serial_test::serial;

const TABLE: &str = "decimal_scale_probe";

/// Creates the column as the generated migration does, writes `value`, reads it back.
async fn round_trip(db: &DatabaseConnection, value: Decimal) -> Decimal {
    let table = Alias::new(TABLE);
    let price = Alias::new("price");
    let drop = Table::drop().table(table.clone()).if_exists().to_owned();
    db.execute(&drop).await.expect("drop probe table");
    db.execute(
        &Table::create()
            .table(table.clone())
            .col(ColumnDef::new(price.clone()).decimal_len(10, 2).not_null())
            .to_owned(),
    )
    .await
    .expect("create probe table");
    db.execute(
        &Query::insert()
            .into_table(table.clone())
            .columns([price.clone()])
            .values_panic([Expr::val(value)])
            .to_owned(),
    )
    .await
    .expect("insert price");
    let row = db
        .query_one(&Query::select().column(price).from(table).to_owned())
        .await
        .expect("select price")
        .expect("one row");
    let read: Decimal = row.try_get("", "price").expect("read price as decimal");
    db.execute(&drop).await.expect("drop probe table");
    read
}

fn price() -> Decimal {
    Decimal::new(999, 2)
}

#[tokio::test]
async fn a_decimal_keeps_its_cents_on_sqlite() {
    let db = db::fresh_db().await;
    assert_eq!(round_trip(&db, price()).await, price());
}

#[tokio::test]
#[serial]
async fn a_decimal_keeps_its_cents_on_postgres() {
    let Some(db) = db_postgres::connect().await else {
        return;
    };
    assert_eq!(round_trip(&db, price()).await, price());
}

#[tokio::test]
#[serial]
async fn a_decimal_keeps_its_cents_on_mariadb() {
    let Some(db) = db_mariadb::connect().await else {
        return;
    };
    assert_eq!(round_trip(&db, price()).await, price());
}
