//! `checkbox` / `multichoice` list fields: no column on the model, a table of
//! their own in the migrations; `i8` / `i16` enums.
use runique::cli::makemigration::scan_entities;
use std::fs;

mod book {
    use runique::prelude::*;

    model! {
        Book,
        table: "list_books",
        pk: id => i32,
        enums: {
            Genre: [Roman, Policier],
            Level: i8 [Low = 1, High = 2],
            Code: i16 [A = -300, B = 300],
        },
        {
            title: text [required],
            level: choice [enum(Level), required],
            code: choice [enum(Code), required],
            genres: checkbox [enum(Genre), required],
            moods: multichoice [enum(Genre)],
        }
    }
}

#[test]
fn a_list_field_is_not_a_column_of_the_model() {
    let names: Vec<String> = book::schema()
        .columns
        .iter()
        .map(|c| c.name.clone())
        .collect();
    assert!(names.contains(&"title".to_string()));
    assert!(
        !names.iter().any(|n| n == "genres" || n == "moods"),
        "{names:?}"
    );
}

#[test]
fn small_integer_enums_store_their_number() {
    assert_eq!(book::Level::High.db_value(), 2i8);
    assert_eq!(book::Code::A.db_value(), -300i16);
}

fn temp_dir(suffix: &str) -> crate::utils::clean_tpm_test::TestTempDir {
    crate::utils::clean_tpm_test::TestTempDir::new("runique_test_lists", suffix)
}

const BOOK: &str = r#"model! { Book, table: "books", pk: id => i64, enums: { Genre: [Roman, Policier], Level: i16 [Low = 1, High = 2] },
    { title: text [required], genres: checkbox [enum(Genre), required], levels: multichoice [enum(Level)] } }"#;

#[test]
fn each_list_field_gets_its_own_table() {
    let dir = temp_dir("tables");
    fs::write(dir.join("book.rs"), BOOK).unwrap();
    let schemas = scan_entities(dir.to_str().unwrap()).unwrap();
    let table = |name: &str| schemas.iter().find(|s| s.table_name == name).unwrap();

    let books = table("books");
    assert!(
        books
            .columns
            .iter()
            .all(|c| c.name != "genres" && c.name != "levels")
    );

    let genres = table("books_genres");
    let col = |name: &str| genres.columns.iter().find(|c| c.name == name).unwrap();
    assert_eq!(
        col("owner_id").col_type,
        "BigInteger",
        "same type as the owner's key"
    );
    assert_eq!(col("value").col_type, "String");
    assert_eq!(col("value").enum_string_values, ["Roman", "Policier"]);
    let fk = &genres.foreign_keys[0];
    assert_eq!(
        (
            fk.from_column.as_str(),
            fk.to_table.as_str(),
            fk.to_column.as_str(),
            fk.on_delete.as_str()
        ),
        ("owner_id", "books", "id", "Cascade")
    );
    let unique = genres.indexes.iter().find(|i| i.unique).unwrap();
    assert_eq!(unique.columns, ["owner_id", "value"]);
    assert!(
        genres
            .indexes
            .iter()
            .any(|i| !i.unique && i.columns == ["value", "owner_id"])
    );

    assert_eq!(table("books_levels").columns[1].col_type, "SmallInteger");
}

#[test]
fn a_list_table_name_already_taken_is_an_error() {
    let dir = temp_dir("collision");
    fs::write(dir.join("book.rs"), BOOK).unwrap();
    fs::write(
        dir.join("other.rs"),
        r#"model! { Other, table: "books_genres", pk: id => i32, { name: text } }"#,
    )
    .unwrap();
    let err = scan_entities(dir.to_str().unwrap()).expect_err("two `books_genres`");
    assert!(format!("{err:#}").contains("books_genres"), "{err:#}");
}

// ── API à l'exécution (SQLite) ───────────────────────────────────────────────

use crate::helpers::db;
use book::{Code, Genre, Level};
use runique::db::ADb;
use runique::sea_orm::{ActiveModelTrait, ActiveValue::Set, EntityTrait, ModelTrait};

const DDL: &str = "CREATE TABLE list_books (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    title TEXT NOT NULL,
    level INTEGER NOT NULL,
    code INTEGER NOT NULL
);
CREATE TABLE list_books_genres (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    owner_id INTEGER NOT NULL REFERENCES list_books(id) ON DELETE CASCADE,
    value TEXT NOT NULL,
    UNIQUE (owner_id, value)
);
CREATE TABLE list_books_moods (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    owner_id INTEGER NOT NULL REFERENCES list_books(id) ON DELETE CASCADE,
    value TEXT NOT NULL,
    UNIQUE (owner_id, value)
);";

async fn insert(conn: &runique::sea_orm::DatabaseConnection, title: &str) -> book::Model {
    book::ActiveModel {
        title: Set(title.into()),
        level: Set(Level::Low),
        code: Set(Code::A),
        ..Default::default()
    }
    .insert(conn)
    .await
    .expect("insert")
}

fn titles(books: Vec<book::Model>) -> Vec<String> {
    let mut t: Vec<String> = books.into_iter().map(|b| b.title).collect();
    t.sort();
    t
}

#[tokio::test]
async fn set_read_and_preload_a_list() {
    let conn = db::fresh_db_with_schema(DDL).await;
    let a = insert(&conn, "a").await;
    let b = insert(&conn, "b").await;

    a.set_genres(&conn, [Genre::Policier, Genre::Roman, Genre::Policier])
        .await
        .unwrap();
    assert_eq!(
        a.genres(&conn).await.unwrap(),
        [Genre::Policier, Genre::Roman],
        "duplicates dropped, order kept"
    );

    a.set_genres(&conn, [Genre::Roman]).await.unwrap();
    assert_eq!(
        a.genres(&conn).await.unwrap(),
        [Genre::Roman],
        "replaced, not added"
    );
    assert!(b.genres(&conn).await.unwrap().is_empty());

    let page = book::Model::load_genres(&conn, &[a.clone(), b.clone()])
        .await
        .unwrap();
    assert_eq!(page[&a.id], [Genre::Roman]);
    assert!(page[&b.id].is_empty());
    assert!(
        book::Model::load_genres(&conn, &[])
            .await
            .unwrap()
            .is_empty()
    );
}

#[tokio::test]
async fn deleting_the_owner_deletes_its_list() {
    let conn = db::fresh_db_with_schema(DDL).await;
    db::exec(&conn, "PRAGMA foreign_keys = ON").await;
    let a = insert(&conn, "a").await;
    a.set_genres(&conn, [Genre::Roman]).await.unwrap();
    a.clone().delete(&conn).await.unwrap();
    assert!(
        book::genres::Entity::find()
            .all(&conn)
            .await
            .unwrap()
            .is_empty()
    );
}

// Written from cargo-mutants survivors (2026-10-07): `one()` was never called.
#[tokio::test]
async fn search_one_returns_none_one_row_or_an_error() {
    let conn = db::fresh_db_with_schema(DDL).await;
    insert(&conn, "seul").await;
    insert(&conn, "double").await;
    insert(&conn, "double").await;
    let adb = ADb::from_connection(conn.clone());

    let none = runique::search!(book::Entity => Title eq "absent")
        .one(&adb)
        .await;
    assert!(matches!(none, Ok(None)), "{none:?}");
    let one = runique::search!(book::Entity => Title eq "seul")
        .one(&adb)
        .await;
    assert_eq!(one.unwrap().map(|b| b.title).as_deref(), Some("seul"));
    let two = runique::search!(book::Entity => Title eq "double")
        .one(&adb)
        .await;
    assert!(two.is_err(), "two rows must be an error, not the first one");
}

#[tokio::test]
async fn search_filters_on_a_list() {
    let conn = db::fresh_db_with_schema(DDL).await;
    let roman = insert(&conn, "roman").await;
    let both = insert(&conn, "both").await;
    let none = insert(&conn, "none").await;
    roman.set_genres(&conn, [Genre::Roman]).await.unwrap();
    both.set_genres(&conn, [Genre::Roman, Genre::Policier])
        .await
        .unwrap();
    let _ = none;
    let adb = ADb::from_connection(conn.clone());

    let has = runique::search!(book::Entity => Genres has Genre::Roman)
        .all(&adb)
        .await
        .unwrap();
    assert_eq!(titles(has), ["both", "roman"]);

    let any = runique::search!(book::Entity => Genres has_any [Genre::Policier])
        .all(&adb)
        .await
        .unwrap();
    assert_eq!(titles(any), ["both"]);

    let all = runique::search!(book::Entity => Genres has_all [Genre::Roman, Genre::Policier])
        .all(&adb)
        .await
        .unwrap();
    assert_eq!(titles(all), ["both"]);

    let not = runique::search!(book::Entity => !Genres has Genre::Roman)
        .all(&adb)
        .await
        .unwrap();
    assert_eq!(titles(not), ["none"]);

    let combined = runique::search!(book::Entity => Genres has Genre::Roman, Title eq "roman")
        .all(&adb)
        .await
        .unwrap();
    assert_eq!(titles(combined), ["roman"]);

    let via_objects = book::Entity::objects
        .filter(book::List::Genres.has(Genre::Policier))
        .all(&adb)
        .await
        .unwrap();
    assert_eq!(titles(via_objects), ["both"]);
}

// Postgres: `Genre` is a native enum type there, so the filters must cast the
// values they compare (`ColumnTrait::is_in` does, from the column's type).
// No `i8` enum here: Postgres can't read one back (refused at compile time
// when `postgres` is the only engine).
mod pg_book {
    use runique::prelude::*;

    model! {
        PgBook,
        table: "pg_list_books",
        pk: id => i32,
        enums: { Genre: [Roman, Policier] },
        {
            title: text [required],
            genres: checkbox [enum(Genre)],
        }
    }
}

#[cfg(feature = "postgres")]
#[tokio::test]
async fn list_on_postgres_with_a_native_enum() {
    use pg_book::Genre as G;
    let Some(conn) = crate::helpers::db_postgres::connect().await else {
        return;
    };
    for sql in [
        "DROP TABLE IF EXISTS pg_list_books_genres",
        "DROP TABLE IF EXISTS pg_list_books",
        "DROP TABLE IF EXISTS list_books_genres",
        "DROP TABLE IF EXISTS list_books",
        "DROP TYPE IF EXISTS genre",
        "CREATE TYPE genre AS ENUM ('Roman', 'Policier')",
        "CREATE TABLE pg_list_books (id SERIAL PRIMARY KEY, title TEXT NOT NULL)",
        "CREATE TABLE pg_list_books_genres (id BIGSERIAL PRIMARY KEY, owner_id INTEGER NOT NULL REFERENCES pg_list_books(id) ON DELETE CASCADE, value genre NOT NULL, UNIQUE (owner_id, value))",
    ] {
        crate::helpers::db_postgres::exec(&conn, sql).await;
    }
    let new = |title: &str| pg_book::ActiveModel {
        title: Set(title.into()),
        ..Default::default()
    };
    let roman = new("roman").insert(&conn).await.unwrap();
    let both = new("both").insert(&conn).await.unwrap();
    roman.set_genres(&conn, [G::Roman]).await.unwrap();
    both.set_genres(&conn, [G::Roman, G::Policier])
        .await
        .unwrap();
    assert_eq!(both.genres(&conn).await.unwrap(), [G::Roman, G::Policier]);

    let adb = ADb::from_connection(conn.clone());
    let names = |rows: Vec<pg_book::Model>| {
        let mut t: Vec<String> = rows.into_iter().map(|b| b.title).collect();
        t.sort();
        t
    };
    let all = runique::search!(pg_book::Entity => Genres has_all [G::Roman, G::Policier])
        .all(&adb)
        .await
        .unwrap();
    assert_eq!(names(all), ["both"]);
    let not = runique::search!(pg_book::Entity => !Genres has G::Policier)
        .all(&adb)
        .await
        .unwrap();
    assert_eq!(names(not), ["roman"]);

    // One round trip: the rows of the query, each with its list.
    use runique::sea_orm::{ColumnTrait, QueryFilter, QueryOrder};
    let _empty = new("empty").insert(&conn).await.unwrap();
    let page = pg_book::List::Genres
        .fetch_with(
            &conn,
            pg_book::Entity::find()
                .filter(pg_book::Column::Title.ne("roman"))
                .order_by_asc(pg_book::Column::Title),
        )
        .await
        .unwrap();
    let page: Vec<(String, Vec<G>)> = page.into_iter().map(|(b, l)| (b.title, l)).collect();
    assert_eq!(
        page,
        [
            ("both".to_string(), vec![G::Roman, G::Policier]),
            ("empty".to_string(), vec![]),
        ]
    );
}

// Needs `fetch_with` (postgres feature) and a SQLite database: `all-databases` builds only.
#[cfg(all(feature = "postgres", feature = "sqlite"))]
#[tokio::test]
async fn fetch_with_says_it_needs_postgres() {
    let conn = db::fresh_db_with_schema(DDL).await;
    let err = book::List::Genres
        .fetch_with(&conn, book::Entity::find())
        .await
        .expect_err("SQLite");
    assert!(err.to_string().contains("load_<field>()"), "{err}");
}

// ── Formulaires et admin ─────────────────────────────────────────────────────

use runique::forms::Forms;
use runique::forms::field::RuniqueForm;
use std::collections::HashMap;

fn admin_form() -> book::AdminForm {
    let mut form = Forms::new("csrf");
    book::AdminForm::register_fields(&mut form);
    book::AdminForm::from_form(form)
}

#[test]
fn the_admin_form_shows_the_list_fields() {
    let form = admin_form();
    let fields = &form.get_form().fields;
    assert_eq!(fields["genres"].field_type(), "checkbox");
    assert_eq!(fields["moods"].field_type(), "select-multiple");
}

#[test]
fn cleaned_enums_reads_a_list_and_cleaned_enum_an_integer_enum() {
    let mut form = admin_form();
    let fields = &mut form.get_form_mut().fields;
    fields
        .get_mut("genres")
        .unwrap()
        .set_value("Roman,Policier");
    fields.get_mut("code").unwrap().set_value("B");
    assert_eq!(
        form.cleaned_enums::<Genre>("genres"),
        [Genre::Roman, Genre::Policier]
    );
    assert_eq!(
        form.cleaned_enum::<Code>("code"),
        Some(Code::B),
        "an i16 enum reads too"
    );
    assert!(
        form.cleaned_enums::<Genre>("moods").is_empty(),
        "nothing chosen"
    );
    assert!(form.cleaned_enums::<Genre>("unknown").is_empty());
}

#[tokio::test]
async fn the_admin_saves_and_reads_back_the_lists() {
    let conn = db::fresh_db_with_schema(DDL).await;
    let a = insert(&conn, "a").await;
    let data = |pairs: &[(&str, &str)]| -> HashMap<String, String> {
        pairs
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect()
    };

    a.admin_save_lists(
        &conn,
        &data(&[("genres", "Roman, Policier"), ("title", "a")]),
    )
    .await
    .unwrap();
    assert_eq!(
        a.genres(&conn).await.unwrap(),
        [Genre::Roman, Genre::Policier]
    );

    a.admin_save_lists(&conn, &data(&[("title", "a")]))
        .await
        .unwrap();
    assert_eq!(
        a.genres(&conn).await.unwrap().len(),
        2,
        "absent from the form: untouched"
    );

    let mut row = serde_json::to_value(&a).unwrap();
    a.admin_list_values(&conn, &mut row).await.unwrap();
    assert_eq!(row["genres"], "Roman,Policier");
    assert_eq!(row["moods"], "");

    a.admin_save_lists(&conn, &data(&[("genres", "")]))
        .await
        .unwrap();
    assert!(
        a.genres(&conn).await.unwrap().is_empty(),
        "nothing checked: cleared"
    );
}
