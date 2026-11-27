//! A value renamed in an enum that two columns use.
//!
//! Under PostgreSQL an enum is one type, shared by every column that uses
//! it: two list fields of a model on the same enum are two tables on one
//! type. The enum is compared once, as the object it is, and its rename is
//! written once, in its own migration, with every place: the tables have
//! nothing to migrate.
//!
//! The last test is why: asked a second time, as it was when each table's
//! migration carried the rename, PostgreSQL refuses it, the former value
//! being gone.
//!
//! Postgres needs `docker compose -f docker-compose.yml up -d` and its URL in
//! `.env.test`; without it, that test returns early.

use crate::helpers::db_postgres;
use runique::sea_orm::ConnectionTrait;
use runique_cli::db::DatabaseEngine;
use runique_cli::migration::scan_entities;
use runique_cli::migration::{
    diff_enums, diff_schemas, enums_of_schemas, generate_enum_file, previous_enums,
};
use sea_query::Alias;
use sea_query::extension::postgres::Type;
use serial_test::serial;
use std::fs;

fn model(first_value: &str) -> String {
    format!(
        r#"model! {{ Book, table: "sh_books", pk: id => i64, enums: {{ Genre: [{first_value}, Policier] }},
    {{ title: text [required], genres: checkbox [enum(Genre)], moods: multichoice [enum(Genre)] }} }}"#
    )
}

fn scanned(suffix: &str, source: &str) -> Vec<runique_cli::migration::ParsedSchema> {
    let dir = crate::utils::clean_tpm_test::TestTempDir::new("runique_test_shared_enum", suffix);
    fs::write(dir.join("book.rs"), source).unwrap();
    scan_entities(dir.to_str().unwrap()).unwrap()
}

// `Roman` becomes `Novel`: neither list table has a migration of its own,
// and the enum has one, which renames the value once and names both places.
#[test]
fn the_rename_is_written_once_for_the_enum_with_every_place() {
    let before = scanned("before", &model("Roman"));
    let after = scanned("after", &model("Novel"));

    for current in &after {
        let previous = before
            .iter()
            .find(|s| s.table_name == current.table_name)
            .expect("the same tables before and after");
        assert!(
            diff_schemas(previous, current).is_empty(),
            "{} has nothing to migrate",
            current.table_name
        );
    }

    let changes = diff_enums(&previous_enums(&before, &[]), &enums_of_schemas(&after));
    assert_eq!(changes.len(), 1, "{changes:?}");
    assert_eq!(changes[0].name, "genre");
    assert_eq!(
        changes[0].renamed_values,
        [("Roman".to_string(), "Novel".to_string())]
    );

    for engine in [
        DatabaseEngine::PostgreSQL,
        DatabaseEngine::MySQL,
        DatabaseEngine::SQLite,
    ] {
        let file = generate_enum_file(&changes[0], &engine);
        let up = file.split("async fn down").next().unwrap();
        // Once on the way up, whatever the number of tables.
        assert_eq!(up.matches("rename_enum_value").count(), 1, "{file}");
        assert!(
            up.contains(r#"rename_enum_value(manager, "genre", "Roman", "Novel", &["#),
            "{file}"
        );
        for place in [
            r#"("sh_books_genres", "value")"#,
            r#"("sh_books_moods", "value")"#,
        ] {
            assert!(up.contains(place), "{place} in {file}");
        }
    }
}

const TYPE: &str = "rq_shared_genre";

fn rename() -> sea_query::extension::postgres::TypeAlterStatement {
    Type::alter()
        .name(Alias::new(TYPE))
        .rename_value(Alias::new("Roman"), Alias::new("Novel"))
        .to_owned()
}

// The statement of the first migration passes; the same statement, as the
// second migration runs it, is refused.
#[tokio::test]
#[serial]
async fn postgres_refuses_the_same_rename_a_second_time() {
    let Some(db) = db_postgres::connect().await else {
        return;
    };
    let drop = Type::drop().if_exists().name(Alias::new(TYPE)).to_owned();
    db.execute(&drop).await.expect("drop the probe type");
    db.execute(
        &Type::create()
            .as_enum(Alias::new(TYPE))
            .values([Alias::new("Roman"), Alias::new("Policier")])
            .to_owned(),
    )
    .await
    .expect("create the probe type");

    let first = db.execute(&rename()).await;
    let second = db.execute(&rename()).await;
    db.execute(&drop).await.expect("drop the probe type");

    assert!(first.is_ok(), "{first:?}");
    let refusal = second.expect_err("the former value is gone").to_string();
    assert!(refusal.contains("Roman"), "{refusal}");
}
