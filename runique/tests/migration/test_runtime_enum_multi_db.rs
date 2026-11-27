//! An enum created, given a value, and a value renamed, through the
//! functions a generated migration calls (`migration::runtime`).
//!
//! Under PostgreSQL the enum is a type of its own, shared by every column
//! that uses it: a value is renamed once, in the type, and every place sees
//! it. Under MySQL and SQLite the column is text: the rows of each place are
//! written. The same calls, on the three engines, leave the same rows.
//!
//! Postgres/MySQL need `docker compose -f docker-compose.yml up -d` and their
//! URL in `.env.test`; without it, those tests return early.

use crate::helpers::{db, db_mysql, db_postgres};
use runique::sea_orm::{ConnectionTrait, DatabaseConnection, DbBackend};
use runique_cli::migration::runtime::{
    add_enum_value, create_enum_type, drop_enum_type, rename_enum_type, rename_enum_value,
};
use sea_orm_migration::SchemaManager;
use sea_query::extension::postgres::Type;
use sea_query::{Alias, ColumnDef, ColumnType, Expr, ExprTrait, IntoIden, Order, Query, Table};
use serial_test::serial;

const TYPE: &str = "rq_rt_genre";
const BOOKS: (&str, &str) = ("rq_rt_books", "genre");
const MOODS: (&str, &str) = ("rq_rt_moods", "value");
const PLACES: [(&str, &str); 2] = [BOOKS, MOODS];
const VALUES: [&str; 2] = ["Roman", "Policier"];

fn is_postgres(db: &DatabaseConnection) -> bool {
    db.get_database_backend() == DbBackend::Postgres
}

async fn clean(db: &DatabaseConnection) {
    for (table, _) in PLACES {
        let drop = Table::drop()
            .table(Alias::new(table))
            .if_exists()
            .to_owned();
        db.execute(&drop).await.expect("drop a test table");
    }
    if is_postgres(db) {
        let drop = Type::drop().if_exists().name(Alias::new(TYPE)).to_owned();
        db.execute(&drop).await.expect("drop the test type");
    }
}

/// The column of an enum as a migration writes it: the type under
/// PostgreSQL, text elsewhere.
fn enum_column(db: &DatabaseConnection, name: &str) -> ColumnDef {
    if is_postgres(db) {
        ColumnDef::new_with_type(
            Alias::new(name),
            ColumnType::Enum {
                name: Alias::new(TYPE).into_iden(),
                variants: VALUES.iter().map(|v| Alias::new(*v).into_iden()).collect(),
            },
        )
    } else {
        ColumnDef::new(Alias::new(name)).string().to_owned()
    }
}

async fn insert(db: &DatabaseConnection, place: (&str, &str), id: i32, value: &str) {
    let stored = if is_postgres(db) {
        Expr::val(value).as_enum(Alias::new(TYPE))
    } else {
        Expr::val(value)
    };
    db.execute(
        &Query::insert()
            .into_table(Alias::new(place.0))
            .columns([Alias::new("id"), Alias::new(place.1)])
            .values_panic([id.into(), stored])
            .to_owned(),
    )
    .await
    .expect("insert a row");
}

/// The two tables, each with a column of the enum, and their rows.
async fn two_places(db: &DatabaseConnection) {
    for place in PLACES {
        db.execute(
            &Table::create()
                .table(Alias::new(place.0))
                .col(
                    ColumnDef::new(Alias::new("id"))
                        .integer()
                        .not_null()
                        .primary_key(),
                )
                .col(enum_column(db, place.1))
                .to_owned(),
        )
        .await
        .expect("create a test table");
    }
    for (id, value) in ["Roman", "Policier", "Roman"].into_iter().enumerate() {
        insert(db, BOOKS, id as i32, value).await;
    }
    for (id, value) in ["Roman", "Policier"].into_iter().enumerate() {
        insert(db, MOODS, id as i32, value).await;
    }
}

/// The values a place holds, by row, read as text on every engine.
async fn stored(db: &DatabaseConnection, place: (&str, &str)) -> Vec<String> {
    let column = Expr::col(Alias::new(place.1));
    let read = if is_postgres(db) {
        column.cast_as(Alias::new("text"))
    } else {
        column
    };
    db.query_all(
        &Query::select()
            .expr(read)
            .from(Alias::new(place.0))
            .order_by(Alias::new("id"), Order::Asc)
            .to_owned(),
    )
    .await
    .expect("read the rows")
    .iter()
    .map(|row| row.try_get_by_index::<String>(0).expect("a text value"))
    .collect()
}

/// The whole life of an enum's values, the same on every engine.
async fn scenario(db: &DatabaseConnection) {
    clean(db).await;
    let manager = SchemaManager::new(db);

    // Created, then asked again: the second call finds it there.
    create_enum_type(&manager, TYPE, &VALUES)
        .await
        .expect("create");
    create_enum_type(&manager, TYPE, &VALUES)
        .await
        .expect("create again");
    two_places(db).await;

    // One call for the enum: every place has the new value, the other rows
    // are as they were.
    rename_enum_value(&manager, TYPE, "Roman", "Novel", &PLACES)
        .await
        .expect("rename");
    assert_eq!(stored(db, BOOKS).await, ["Novel", "Policier", "Novel"]);
    assert_eq!(stored(db, MOODS).await, ["Novel", "Policier"]);

    // The former value is free again, the new one is the stored one.
    insert(db, BOOKS, 10, "Novel").await;
    assert_eq!(stored(db, BOOKS).await.last().unwrap(), "Novel");

    // What `down` runs: the same call, the other way round.
    rename_enum_value(&manager, TYPE, "Novel", "Roman", &PLACES)
        .await
        .expect("rename back");
    assert_eq!(
        stored(db, BOOKS).await,
        ["Roman", "Policier", "Roman", "Roman"]
    );
    assert_eq!(stored(db, MOODS).await, ["Roman", "Policier"]);

    // A value added, then asked again; it can then be stored.
    add_enum_value(&manager, TYPE, "Essai").await.expect("add");
    add_enum_value(&manager, TYPE, "Essai")
        .await
        .expect("add again");
    insert(db, MOODS, 10, "Essai").await;
    assert_eq!(stored(db, MOODS).await, ["Roman", "Policier", "Essai"]);

    // Still used by its two places: the type stays. Once they are gone, it
    // goes.
    drop_enum_type(&manager, TYPE)
        .await
        .expect("drop while used");
    assert_eq!(stored(db, BOOKS).await.len(), 4);
    for (table, _) in PLACES {
        let drop = Table::drop().table(Alias::new(table)).to_owned();
        db.execute(&drop).await.expect("drop a test table");
    }
    drop_enum_type(&manager, TYPE).await.expect("drop");
    // Created again from nothing: it was really gone under PostgreSQL, and
    // there was never anything to find elsewhere.
    create_enum_type(&manager, TYPE, &["Other"])
        .await
        .expect("create after the drop");
    if is_postgres(db) {
        let again = Type::create()
            .as_enum(Alias::new(TYPE))
            .values([Alias::new("Other")])
            .to_owned();
        assert!(
            db.execute(&again).await.is_err(),
            "the type exists, created by the runtime"
        );
    }
    clean(db).await;
}

#[tokio::test]
async fn sqlite_an_enum_s_values_live_in_the_rows() {
    scenario(&db::fresh_db().await).await;
}

#[tokio::test]
#[serial]
async fn postgres_an_enum_s_values_live_in_its_type() {
    let Some(db) = db_postgres::connect().await else {
        return;
    };
    scenario(&db).await;
}

#[tokio::test]
#[serial]
async fn mysql_an_enum_s_values_live_in_the_rows() {
    let Some(db) = db_mysql::connect().await else {
        return;
    };
    scenario(&db).await;
}

// Under PostgreSQL the rename is asked once for the enum, never once per
// place: the type is shared, and a second rename finds the former value gone.
#[tokio::test]
#[serial]
async fn postgres_a_value_is_renamed_once_for_the_enum() {
    let Some(db) = db_postgres::connect().await else {
        return;
    };
    clean(&db).await;
    let manager = SchemaManager::new(&db);
    create_enum_type(&manager, TYPE, &VALUES)
        .await
        .expect("create");
    two_places(&db).await;

    // No place is needed there: the type carries the change to all of them.
    rename_enum_value(&manager, TYPE, "Roman", "Novel", &[])
        .await
        .expect("rename");
    assert_eq!(stored(&db, BOOKS).await, ["Novel", "Policier", "Novel"]);
    assert_eq!(stored(&db, MOODS).await, ["Novel", "Policier"]);

    let second = rename_enum_value(&manager, TYPE, "Roman", "Novel", &[]).await;
    clean(&db).await;
    let refusal = second.expect_err("the former value is gone").to_string();
    assert!(refusal.contains("Roman"), "{refusal}");
}

// Under MySQL and SQLite a place that is not given keeps its rows: the
// places are what makes the rename reach every column.
#[tokio::test]
async fn sqlite_a_place_left_out_keeps_the_former_value() {
    let db = db::fresh_db().await;
    clean(&db).await;
    two_places(&db).await;

    rename_enum_value(&SchemaManager::new(&db), TYPE, "Roman", "Novel", &[BOOKS])
        .await
        .expect("rename");
    assert_eq!(stored(&db, BOOKS).await, ["Novel", "Policier", "Novel"]);
    assert_eq!(stored(&db, MOODS).await, ["Roman", "Policier"]);
}

// ── An enum renamed ──────────────────────────────────────────────────────────

const RENAMED: &str = "rq_rt_category";

// Under PostgreSQL the type takes its new name in place: the columns follow,
// their rows untouched, and the former name is free. The same call the other
// way round is what `down` runs.
#[tokio::test]
#[serial]
async fn postgres_a_renamed_enum_keeps_its_columns_and_their_rows() {
    let Some(db) = db_postgres::connect().await else {
        return;
    };
    clean(&db).await;
    let drop_renamed = Type::drop()
        .if_exists()
        .name(Alias::new(RENAMED))
        .cascade()
        .to_owned();
    db.execute(&drop_renamed)
        .await
        .expect("drop the renamed type");
    let manager = SchemaManager::new(&db);
    create_enum_type(&manager, TYPE, &VALUES)
        .await
        .expect("create");
    two_places(&db).await;

    rename_enum_type(&manager, TYPE, RENAMED)
        .await
        .expect("rename the type");

    assert_eq!(stored(&db, BOOKS).await, ["Roman", "Policier", "Roman"]);
    assert_eq!(stored(&db, MOODS).await, ["Roman", "Policier"]);
    // The former name is free: a type can be created under it again.
    let former = Type::create()
        .as_enum(Alias::new(TYPE))
        .values([Alias::new("Other")])
        .to_owned();
    let free = db.execute(&former).await;
    let drop_former = Type::drop().if_exists().name(Alias::new(TYPE)).to_owned();
    db.execute(&drop_former).await.expect("drop the probe type");
    assert!(free.is_ok(), "{free:?}");
    // A value of the type, under its new name, is still one of the column.
    rename_enum_value(&manager, RENAMED, "Roman", "Novel", &[])
        .await
        .expect("rename a value of the renamed type");
    assert_eq!(stored(&db, BOOKS).await, ["Novel", "Policier", "Novel"]);

    rename_enum_type(&manager, RENAMED, TYPE)
        .await
        .expect("rename the type back");
    assert_eq!(stored(&db, MOODS).await, ["Novel", "Policier"]);
    clean(&db).await;
}

// Renamed a second time, the former name is gone: the engine says so.
#[tokio::test]
#[serial]
async fn postgres_renaming_an_enum_that_is_not_there_is_an_error() {
    let Some(db) = db_postgres::connect().await else {
        return;
    };
    clean(&db).await;
    let refused = rename_enum_type(&SchemaManager::new(&db), TYPE, RENAMED).await;
    assert!(refused.is_err(), "no type of that name to rename");
}

// Where the column is text, no object carries the name of an enum: the rows
// are as they were, whatever the engine.
async fn text_rename_changes_nothing(db: &DatabaseConnection) {
    clean(db).await;
    two_places(db).await;

    rename_enum_type(&SchemaManager::new(db), TYPE, RENAMED)
        .await
        .expect("nothing to rename");

    assert_eq!(stored(db, BOOKS).await, ["Roman", "Policier", "Roman"]);
    assert_eq!(stored(db, MOODS).await, ["Roman", "Policier"]);
    clean(db).await;
}

#[tokio::test]
async fn sqlite_renaming_an_enum_leaves_the_rows() {
    text_rename_changes_nothing(&db::fresh_db().await).await;
}

#[tokio::test]
#[serial]
async fn mysql_renaming_an_enum_leaves_the_rows() {
    let Some(db) = db_mysql::connect().await else {
        return;
    };
    text_rename_changes_nothing(&db).await;
}
