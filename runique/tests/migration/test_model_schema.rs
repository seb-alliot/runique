// Tests pour ModelSchema et SchemaDiff

use runique::forms::Forms;
use runique::migration::{
    column::ColumnDef,
    foreign_key::ForeignKeyDef,
    hooks::HooksDef,
    index::IndexDef,
    primary_key::PrimaryKeyDef,
    schema::{ModelSchema, SchemaDiff},
};

// ═══════════════════════════════════════════════════════════════
// ModelSchema::new() — conversion PascalCase → snake_case
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_schema_new_pascal_case_simple() {
    let s = ModelSchema::new("User");
    assert_eq!(s.model_name, "User");
    assert_eq!(s.table_name, "user");
}

#[test]
fn test_schema_new_pascal_case_compose() {
    let s = ModelSchema::new("BlogPost");
    assert_eq!(s.model_name, "BlogPost");
    assert_eq!(s.table_name, "blog_post");
}

#[test]
fn test_schema_new_defauts() {
    let s = ModelSchema::new("Article");
    assert!(s.primary_key.is_none());
    assert!(s.columns.is_empty());
    assert!(s.foreign_keys.is_empty());
    assert!(s.indexes.is_empty());
    assert!(s.hooks.is_none());
    assert!(s.schema.is_none());
}

// ═══════════════════════════════════════════════════════════════
// Builders — table_name, schema
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_schema_table_name_override() {
    let s = ModelSchema::new("User").table_name("custom_users");
    assert_eq!(s.table_name, "custom_users");
}

#[test]
fn test_schema_set_schema() {
    let s = ModelSchema::new("User").schema("public");
    assert_eq!(s.schema.as_deref(), Some("public"));
}

// ═══════════════════════════════════════════════════════════════
// Builders — primary_key, column, foreign_key, index, hooks
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_schema_primary_key() {
    let s = ModelSchema::new("User").primary_key(PrimaryKeyDef::new("id"));
    assert!(s.primary_key.is_some());
    assert_eq!(s.primary_key.unwrap().name, "id");
}

#[test]
fn test_schema_column_ajout() {
    let s = ModelSchema::new("User").column(ColumnDef::new("username").string());
    assert_eq!(s.columns.len(), 1);
    assert_eq!(s.columns[0].name, "username");
}

#[test]
fn test_schema_multi_columns() {
    let s = ModelSchema::new("Post")
        .column(ColumnDef::new("title").string())
        .column(ColumnDef::new("body").text());
    assert_eq!(s.columns.len(), 2);
}

#[test]
fn test_schema_foreign_key_ajout() {
    let s = ModelSchema::new("Post").foreign_key(ForeignKeyDef::new("user_id").references("users"));
    assert_eq!(s.foreign_keys.len(), 1);
    assert_eq!(s.foreign_keys[0].from_column, "user_id");
}

#[test]
fn test_schema_index_ajout() {
    let s = ModelSchema::new("User").index(IndexDef::new(vec!["email"]).unique());
    assert_eq!(s.indexes.len(), 1);
    assert!(s.indexes[0].unique);
}

#[test]
fn test_schema_hooks_ajout() {
    let s = ModelSchema::new("User").hooks(HooksDef::new().before_save(0, "handler"));
    assert!(s.hooks.is_some());
    assert_eq!(s.hooks.unwrap().hooks.len(), 1);
}

// ═══════════════════════════════════════════════════════════════
// build()
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_schema_build_sans_pk_retourne_err() {
    let result = ModelSchema::new("User").build();
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("missing primary key"));
}

#[test]
fn test_schema_build_avec_pk_retourne_ok() {
    let result = ModelSchema::new("User")
        .primary_key(PrimaryKeyDef::new("id"))
        .build();
    assert!(result.is_ok());
    assert_eq!(result.unwrap().model_name, "User");
}

// ═══════════════════════════════════════════════════════════════
// diff()
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_schema_diff_identiques_est_vide() {
    let s1 = ModelSchema::new("User").column(ColumnDef::new("name").string());
    let s2 = ModelSchema::new("User").column(ColumnDef::new("name").string());
    let diff = s1.diff(&s2);
    assert!(diff.is_empty());
}

#[test]
fn test_schema_diff_colonne_ajoutee() {
    let old = ModelSchema::new("User").column(ColumnDef::new("name").string());
    let new = ModelSchema::new("User")
        .column(ColumnDef::new("name").string())
        .column(ColumnDef::new("email").string());
    let diff = old.diff(&new);
    assert!(!diff.is_empty());
    assert_eq!(diff.added_columns.len(), 1);
    assert_eq!(diff.added_columns[0].name, "email");
    assert!(diff.dropped_columns.is_empty());
}

#[test]
fn test_schema_diff_colonne_supprimee() {
    let old = ModelSchema::new("User")
        .column(ColumnDef::new("name").string())
        .column(ColumnDef::new("email").string());
    let new = ModelSchema::new("User").column(ColumnDef::new("name").string());
    let diff = old.diff(&new);
    assert!(!diff.is_empty());
    assert_eq!(diff.dropped_columns.len(), 1);
    assert_eq!(diff.dropped_columns[0], "email");
    assert!(diff.added_columns.is_empty());
}

// ═══════════════════════════════════════════════════════════════
// SchemaDiff
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_schema_diff_new_est_vide() {
    let diff = SchemaDiff::new("users");
    assert_eq!(diff.table_name, "users");
    assert!(diff.is_empty());
    assert!(diff.added_columns.is_empty());
    assert!(diff.dropped_columns.is_empty());
    assert!(diff.modified_columns.is_empty());
}

#[test]
fn test_schema_clone() {
    let s = ModelSchema::new("User")
        .primary_key(PrimaryKeyDef::new("id"))
        .column(ColumnDef::new("name").string());
    let cloned = s.clone();
    assert_eq!(cloned.model_name, "User");
    assert_eq!(cloned.table_name, "user");
    assert_eq!(cloned.columns.len(), 1);
    assert!(cloned.primary_key.is_some());
}

// ═══════════════════════════════════════════════════════════════
// fill_form()
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_schema_fill_form_all_fields() {
    let s = ModelSchema::new("User")
        .primary_key(PrimaryKeyDef::new("id"))
        .column(ColumnDef::new("username").string())
        .column(ColumnDef::new("email").string());
    let mut form = Forms::new("dummy_token");
    let before = form.fields.len();
    s.fill_form(&mut form, None, None);
    // 2 colonnes ajoutées (PK exclue automatiquement)
    assert_eq!(form.fields.len() - before, 2);
}

#[test]
fn test_schema_fill_form_with_exclude() {
    let s = ModelSchema::new("User")
        .primary_key(PrimaryKeyDef::new("id"))
        .column(ColumnDef::new("username").string())
        .column(ColumnDef::new("email").string())
        .column(ColumnDef::new("password").string());
    let mut form = Forms::new("dummy_token");
    let before = form.fields.len();
    s.fill_form(&mut form, None, Some(&["password"]));
    assert_eq!(form.fields.len() - before, 2);
}

#[test]
fn test_schema_fill_form_with_whitelist() {
    let s = ModelSchema::new("User")
        .primary_key(PrimaryKeyDef::new("id"))
        .column(ColumnDef::new("username").string())
        .column(ColumnDef::new("email").string())
        .column(ColumnDef::new("bio").text());
    let mut form = Forms::new("dummy_token");
    let before = form.fields.len();
    s.fill_form(&mut form, Some(&["username", "email"]), None);
    assert_eq!(form.fields.len() - before, 2);
}

// The whitelist takes the columns it names, not the same number of others.
#[test]
fn test_schema_fill_form_whitelist_takes_the_named_columns() {
    let s = ModelSchema::new("User")
        .primary_key(PrimaryKeyDef::new("id"))
        .column(ColumnDef::new("username").string())
        .column(ColumnDef::new("email").string())
        .column(ColumnDef::new("bio").text());
    let mut form = Forms::new("dummy_token");
    s.fill_form(&mut form, Some(&["bio"]), None);
    assert!(form.fields.contains_key("bio"));
    assert!(!form.fields.contains_key("username"));
}
