//! An enum column takes a value added by a later migration, on every engine.
//! `makemigrations` writes the column as a Postgres enum type on Postgres and
//! a plain string elsewhere (see `tests_pipeline.rs`), and a value added
//! later as `ALTER TYPE … ADD VALUE` on Postgres, nothing elsewhere. A native
//! `ENUM(...)` on MySQL/MariaDB would keep its first list and refuse the new
//! value.
//!
//! Postgres/MariaDB need `docker compose -f docker-compose.yml up -d` and their
//! URL in `.env.test`; without it, those tests return early.

use crate::helpers::{db, db_mariadb, db_postgres};
use runique::sea_orm::{ConnectionTrait, DatabaseConnection, DbBackend};
use sea_query::extension::postgres::Type;
use sea_query::{Alias, ColumnDef, ColumnType, Expr, ExprTrait, IntoIden, Query, Table};
use serial_test::serial;

const TABLE: &str = "enum_column_probe";
const TYPE: &str = "enum_column_probe_status";

/// Creates the column as the generated migration does, adds `Archived` as a
/// later migration does, writes it, reads it back.
async fn added_value_round_trip(db: &DatabaseConnection) -> String {
    let postgres = db.get_database_backend() == DbBackend::Postgres;
    let table = Alias::new(TABLE);
    let status = Alias::new("status");
    let drop = Table::drop().table(table.clone()).if_exists().to_owned();
    db.execute(&drop).await.expect("drop probe table");
    if postgres {
        db.execute(&Type::drop().if_exists().name(Alias::new(TYPE)).to_owned())
            .await
            .expect("drop probe type");
        db.execute(
            &Type::create()
                .as_enum(Alias::new(TYPE))
                .values([Alias::new("Draft"), Alias::new("Published")])
                .to_owned(),
        )
        .await
        .expect("create probe type");
    }

    let column_type = if postgres {
        ColumnType::Enum {
            name: Alias::new(TYPE).into_iden(),
            variants: vec![
                Alias::new("Draft").into_iden(),
                Alias::new("Published").into_iden(),
            ],
        }
    } else {
        ColumnType::String(Default::default())
    };
    db.execute(
        &Table::create()
            .table(table.clone())
            .col(
                ColumnDef::new_with_type(status.clone(), column_type)
                    .not_null()
                    .default("Draft"),
            )
            .to_owned(),
    )
    .await
    .expect("create probe table");

    if postgres {
        db.execute(
            &Type::alter()
                .name(Alias::new(TYPE))
                .add_value(Alias::new("Archived"))
                .if_not_exists()
                .to_owned(),
        )
        .await
        .expect("add the enum value");
    }

    let value = if postgres {
        Expr::val("Archived").cast_as(Alias::new(TYPE))
    } else {
        Expr::val("Archived")
    };
    db.execute(
        &Query::insert()
            .into_table(table.clone())
            .columns([status.clone()])
            .values_panic([value])
            .to_owned(),
    )
    .await
    .expect("insert the added value");

    let read = if postgres {
        Expr::col(status).cast_as(Alias::new("text"))
    } else {
        Expr::col(status)
    };
    let row = db
        .query_one(
            &Query::select()
                .expr_as(read, Alias::new("status"))
                .from(table)
                .to_owned(),
        )
        .await
        .expect("select status")
        .expect("one row");
    let got: String = row.try_get("", "status").expect("read status");

    db.execute(&drop).await.expect("drop probe table");
    if postgres {
        db.execute(&Type::drop().if_exists().name(Alias::new(TYPE)).to_owned())
            .await
            .expect("drop probe type");
    }
    got
}

#[tokio::test]
async fn an_added_enum_value_is_stored_on_sqlite() {
    let db = db::fresh_db().await;
    assert_eq!(added_value_round_trip(&db).await, "Archived");
}

#[tokio::test]
#[serial]
async fn an_added_enum_value_is_stored_on_postgres() {
    let Some(db) = db_postgres::connect().await else {
        return;
    };
    assert_eq!(added_value_round_trip(&db).await, "Archived");
}

#[tokio::test]
#[serial]
async fn an_added_enum_value_is_stored_on_mariadb() {
    let Some(db) = db_mariadb::connect().await else {
        return;
    };
    assert_eq!(added_value_round_trip(&db).await, "Archived");
}
