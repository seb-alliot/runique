//! Tests — Migration Diff + update_migration_lib
//! Couvre : diff_schemas, db_columns, Changes::is_empty, update_migration_lib

use runique::migration::utils::diff::{db_columns, diff_schemas};
use runique::migration::utils::types::{ParsedColumn, ParsedSchema};
use runique::utils::cli::makemigration::update_migration_lib;

// ── Helpers ───────────────────────────────────────────────────────────────────

fn col(name: &str, col_type: &str) -> ParsedColumn {
    ParsedColumn {
        name: name.to_string(),
        col_type: col_type.to_string(),
        ..ParsedColumn::default()
    }
}

fn col_nullable(name: &str, col_type: &str) -> ParsedColumn {
    ParsedColumn {
        nullable: true,
        ..col(name, col_type)
    }
}

fn col_ignored(name: &str, col_type: &str) -> ParsedColumn {
    ParsedColumn {
        ignored: true,
        ..col(name, col_type)
    }
}

fn schema(table: &str, pk_name: &str, columns: Vec<ParsedColumn>) -> ParsedSchema {
    ParsedSchema {
        table_name: table.to_string(),
        primary_key: Some(col(pk_name, "i32")),
        columns,
        foreign_keys: vec![],
        indexes: vec![],
    }
}

// ── db_columns ────────────────────────────────────────────────────────────────

#[test]
fn test_db_columns_excludes_pk() {
    let s = schema("users", "id", vec![col("id", "i32"), col("name", "String")]);
    let cols = db_columns(&s);
    assert!(!cols.iter().any(|c| c.name == "id"), "PK doit être exclue");
    assert!(cols.iter().any(|c| c.name == "name"));
}

#[test]
fn test_db_columns_excludes_ignored() {
    let s = schema(
        "users",
        "id",
        vec![col("name", "String"), col_ignored("created_at", "DateTime")],
    );
    let cols = db_columns(&s);
    assert!(
        !cols.iter().any(|c| c.name == "created_at"),
        "ignored doit être exclu"
    );
}

#[test]
fn test_db_columns_returns_normal_columns() {
    let s = schema(
        "users",
        "id",
        vec![col("username", "String"), col("email", "String")],
    );
    let cols = db_columns(&s);
    assert_eq!(cols.len(), 2);
}

#[test]
fn test_db_columns_empty_schema() {
    let s = schema("users", "id", vec![]);
    assert!(db_columns(&s).is_empty());
}

// ── Changes::is_empty ─────────────────────────────────────────────────────────

#[test]
fn test_changes_is_empty_when_no_changes() {
    let s = schema("blog", "id", vec![col("title", "String")]);
    let changes = diff_schemas(&s, &s);
    assert!(changes.is_empty());
}

#[test]
fn test_changes_not_empty_for_new_table() {
    let s = schema("blog", "id", vec![col("title", "String")]);
    let empty = ParsedSchema {
        table_name: "blog".to_string(),
        primary_key: Some(col("id", "i32")),
        columns: vec![],
        foreign_keys: vec![],
        indexes: vec![],
    };
    let changes = diff_schemas(&empty, &s);
    assert!(!changes.is_empty());
}

// ── diff_schemas ──────────────────────────────────────────────────────────────

#[test]
fn test_diff_no_changes_identical_schemas() {
    let s = schema(
        "blog",
        "id",
        vec![col("title", "String"), col("views", "i32")],
    );
    let changes = diff_schemas(&s, &s);
    assert!(changes.added_columns.is_empty());
    assert!(changes.dropped_columns.is_empty());
    assert!(changes.modified_columns.is_empty());
}

#[test]
fn test_diff_detects_added_column() {
    let prev = schema("blog", "id", vec![col("title", "String")]);
    let curr = schema(
        "blog",
        "id",
        vec![col("title", "String"), col("views", "i32")],
    );
    let changes = diff_schemas(&prev, &curr);
    assert_eq!(changes.added_columns.len(), 1);
    assert_eq!(changes.added_columns[0].name, "views");
}

#[test]
fn test_diff_detects_dropped_column() {
    let prev = schema(
        "blog",
        "id",
        vec![col("title", "String"), col("views", "i32")],
    );
    let curr = schema("blog", "id", vec![col("title", "String")]);
    let changes = diff_schemas(&prev, &curr);
    assert_eq!(changes.dropped_columns.len(), 1);
    assert_eq!(changes.dropped_columns[0].name, "views");
}

#[test]
fn test_diff_detects_type_change() {
    let prev = schema("blog", "id", vec![col("views", "i32")]);
    let curr = schema("blog", "id", vec![col("views", "i64")]);
    let changes = diff_schemas(&prev, &curr);
    assert_eq!(changes.modified_columns.len(), 1);
    let (old, new) = &changes.modified_columns[0];
    assert_eq!(old.col_type, "i32");
    assert_eq!(new.col_type, "i64");
}

#[test]
fn test_diff_detects_nullable_change() {
    let prev = schema("blog", "id", vec![col("summary", "String")]);
    let curr = schema("blog", "id", vec![col_nullable("summary", "String")]);
    let changes = diff_schemas(&prev, &curr);
    assert_eq!(changes.modified_columns.len(), 1);
}

#[test]
fn test_diff_multiple_changes() {
    let prev = schema(
        "blog",
        "id",
        vec![col("title", "String"), col("old_field", "String")],
    );
    let curr = schema(
        "blog",
        "id",
        vec![col("title", "String"), col("new_field", "i32")],
    );
    let changes = diff_schemas(&prev, &curr);
    assert_eq!(changes.added_columns.len(), 1, "new_field ajouté");
    assert_eq!(changes.dropped_columns.len(), 1, "old_field supprimé");
    assert!(!changes.is_empty());
}

// ── enum_renames dans diff_schemas ───────────────────────────────────────────

fn col_enum(name: &str, enum_name: &str, values: &[&str]) -> ParsedColumn {
    ParsedColumn {
        enum_name: Some(enum_name.to_string()),
        enum_string_values: values.iter().map(|s| s.to_string()).collect(),
        ..col(name, "String")
    }
}

#[test]
fn test_diff_enum_values_identiques_pas_de_rename() {
    let vals = &["Draft", "Published"];
    let prev = schema("articles", "id", vec![col_enum("status", "Status", vals)]);
    let curr = schema("articles", "id", vec![col_enum("status", "Status", vals)]);
    let changes = diff_schemas(&prev, &curr);
    assert!(
        changes.enum_renames.is_empty(),
        "Valeurs identiques → pas de rename"
    );
}

#[test]
fn test_diff_enum_values_changees_genere_rename() {
    let prev = schema(
        "articles",
        "id",
        vec![col_enum("status", "Status", &["Ajoute", "Fix"])],
    );
    let curr = schema(
        "articles",
        "id",
        vec![col_enum("status", "Status", &["Ajouté", "Fix"])],
    );
    let changes = diff_schemas(&prev, &curr);
    assert_eq!(changes.enum_renames.len(), 1, "Un rename attendu");
    let (col, old_val, new_val) = &changes.enum_renames[0];
    assert_eq!(col, "status");
    assert_eq!(old_val, "Ajoute");
    assert_eq!(new_val, "Ajouté");
}

#[test]
fn test_diff_enum_plusieurs_valeurs_changees() {
    let prev = schema(
        "articles",
        "id",
        vec![col_enum("status", "Status", &["v1", "v2", "v3"])],
    );
    let curr = schema(
        "articles",
        "id",
        vec![col_enum("status", "Status", &["V1", "V2", "v3"])],
    );
    let changes = diff_schemas(&prev, &curr);
    assert_eq!(changes.enum_renames.len(), 2);
}

#[test]
fn test_diff_enum_renames_in_changes_not_empty() {
    let prev = schema(
        "articles",
        "id",
        vec![col_enum("status", "Status", &["old"])],
    );
    let curr = schema(
        "articles",
        "id",
        vec![col_enum("status", "Status", &["new"])],
    );
    let changes = diff_schemas(&prev, &curr);
    assert!(
        !changes.is_empty(),
        "Des renames → changes.is_empty() == false"
    );
}

// ── update_migration_lib ──────────────────────────────────────────────────────

fn tmp_dir(suffix: &str) -> crate::utils::clean_tpm_test::TestTempDir {
    crate::utils::clean_tpm_test::TestTempDir::new("runique_test", suffix)
}

#[test]
fn test_update_migration_lib_creates_lib_file() {
    let path = tmp_dir("create_lib");
    let module = "m20260101000000_create_users_table";

    update_migration_lib(path.as_str(), module).unwrap();

    let content = std::fs::read_to_string(format!("{}/lib.rs", path)).unwrap();
    assert!(content.contains(&format!("mod {};", module)));
    assert!(content.contains(&format!("Box::new({}::Migration)", module)));
    assert!(content.contains("use sea_orm_migration::prelude::*;"));
}

#[test]
fn test_update_migration_lib_appends_to_existing() {
    let path = tmp_dir("append_lib");
    let m1 = "m20260101000000_create_users_table";
    let m2 = "m20260102000000_create_blog_table";

    update_migration_lib(path.as_str(), m1).unwrap();
    update_migration_lib(path.as_str(), m2).unwrap();

    let content = std::fs::read_to_string(format!("{}/lib.rs", path)).unwrap();
    assert!(content.contains(&format!("mod {};", m1)));
    assert!(content.contains(&format!("mod {};", m2)));
}

#[test]
fn test_update_migration_lib_idempotent() {
    let path = tmp_dir("idempotent_lib");
    let module = "m20260101000000_create_users_table";

    update_migration_lib(path.as_str(), module).unwrap();
    update_migration_lib(path.as_str(), module).unwrap();

    let content = std::fs::read_to_string(format!("{}/lib.rs", path)).unwrap();
    let count = content.matches(&format!("mod {};", module)).count();
    assert_eq!(count, 1, "Le module ne doit apparaitre qu'une fois");
}
