//! Probe: a value renamed in an enum that two columns use.
//!
//! Under PostgreSQL an enum is one type, shared by every column that uses
//! it: two list fields of a model on the same enum are two tables on one
//! type. The rename of a value is computed column by column, so each table's
//! migration carries it. These tests confirm both halves of what follows:
//! the generator writes the rename once per table, and PostgreSQL refuses it
//! the second time, the former value being gone.
//!
//! They describe what happens today, not what should: once the generator
//! writes the rename once per type, the first test changes side.
//!
//! Postgres needs `docker compose -f docker-compose.yml up -d` and its URL in
//! `.env.test`; without it, that test returns early.

use crate::helpers::db_postgres;
use runique::sea_orm::ConnectionTrait;
use runique_cli::migration::scan_entities;
use runique_cli::migration::{diff_schemas, generate_alter_file};
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

const RENAME: &str = r#".rename_value(Alias::new("Roman"), Alias::new("Novel"))"#;

// `Roman` becomes `Novel`: both list tables get a migration, and each one
// renames the value in the type they share.
#[test]
fn the_rename_is_written_once_per_table_using_the_enum() {
    let before = scanned("before", &model("Roman"));
    let after = scanned("after", &model("Novel"));

    let mut renaming: Vec<String> = Vec::new();
    for current in &after {
        let previous = before
            .iter()
            .find(|s| s.table_name == current.table_name)
            .expect("the same tables before and after");
        let changes = diff_schemas(previous, current);
        if !changes.is_empty() && generate_alter_file(&changes).contains(RENAME) {
            renaming.push(current.table_name.clone());
        }
    }
    renaming.sort();

    assert_eq!(renaming, ["sh_books_genres", "sh_books_moods"]);
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
