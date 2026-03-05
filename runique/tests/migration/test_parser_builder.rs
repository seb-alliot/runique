//! Tests — Migration Parser (DSL model!)
//! Couvre : parse_schema_from_source (parser_builder)

use runique::migration::utils::parser_builder::parse_schema_from_source;

// ── Source DSL valide ─────────────────────────────────────────────────────────

fn blog_source() -> &'static str {
    r#"
    use runique::prelude::*;
    model! {
        Blog,
        table: "blog",
        pk: id => i32,
        fields: {
            title: String,
            summary: String [nullable],
            views: i32,
            published: bool [nullable],
        }
    }
    "#
}

fn users_source() -> &'static str {
    r#"
    use runique::prelude::*;
    model! {
        User,
        table: "users",
        pk: id => i64,
        fields: {
            username: String [unique],
            email: String [unique],
            is_active: bool,
            created_at: DateTime [auto_now],
            updated_at: DateTime [auto_now_update],
        }
    }
    "#
}

// ── Parsing réussi ────────────────────────────────────────────────────────────

#[test]
fn test_parse_schema_returns_some() {
    let result = parse_schema_from_source(blog_source());
    assert!(
        result.is_some(),
        "Le parser doit retourner Some pour un model! valide"
    );
}

#[test]
fn test_parse_schema_table_name() {
    let schema = parse_schema_from_source(blog_source()).unwrap();
    assert_eq!(schema.table_name, "blog");
}

#[test]
fn test_parse_schema_primary_key() {
    let schema = parse_schema_from_source(blog_source()).unwrap();
    let pk = schema.primary_key.as_ref().unwrap();
    assert_eq!(pk.name, "id");
}

#[test]
fn test_parse_schema_field_count() {
    let schema = parse_schema_from_source(blog_source()).unwrap();
    // 4 champs: title, summary, views, published
    assert_eq!(schema.columns.len(), 4);
}

#[test]
fn test_parse_schema_field_names() {
    let schema = parse_schema_from_source(blog_source()).unwrap();
    let names: Vec<&str> = schema.columns.iter().map(|c| c.name.as_str()).collect();
    assert!(names.contains(&"title"));
    assert!(names.contains(&"summary"));
    assert!(names.contains(&"views"));
    assert!(names.contains(&"published"));
}

// ── Options des champs ────────────────────────────────────────────────────────

#[test]
fn test_parse_nullable_field() {
    let schema = parse_schema_from_source(blog_source()).unwrap();
    let summary = schema.columns.iter().find(|c| c.name == "summary").unwrap();
    assert!(summary.nullable, "summary doit être nullable");
}

#[test]
fn test_parse_non_nullable_field() {
    let schema = parse_schema_from_source(blog_source()).unwrap();
    let title = schema.columns.iter().find(|c| c.name == "title").unwrap();
    assert!(!title.nullable, "title ne doit pas être nullable");
}

#[test]
fn test_parse_unique_field() {
    let schema = parse_schema_from_source(users_source()).unwrap();
    let username = schema
        .columns
        .iter()
        .find(|c| c.name == "username")
        .unwrap();
    assert!(username.unique, "username doit être unique");
}

#[test]
fn test_parse_auto_now_becomes_datetime_and_ignored() {
    let schema = parse_schema_from_source(users_source()).unwrap();
    let created_at = schema
        .columns
        .iter()
        .find(|c| c.name == "created_at")
        .unwrap();
    assert_eq!(created_at.col_type, "DateTime");
    assert!(
        !created_at.ignored,
        "auto_now ne doit plus marquer le champ comme ignored"
    );
}

// ── Source invalide / vide ────────────────────────────────────────────────────

#[test]
fn test_parse_empty_source_returns_none() {
    let result = parse_schema_from_source("");
    assert!(result.is_none(), "Source vide doit retourner None");
}

#[test]
fn test_parse_no_model_macro_returns_none() {
    let source = r#"
        pub struct Foo {
            pub id: i32,
            pub name: String,
        }
    "#;
    let result = parse_schema_from_source(source);
    assert!(result.is_none(), "Pas de macro model! → None");
}

// ── Isolation entre tables ────────────────────────────────────────────────────

#[test]
fn test_parse_different_tables_independent() {
    let blog = parse_schema_from_source(blog_source()).unwrap();
    let users = parse_schema_from_source(users_source()).unwrap();
    assert_ne!(blog.table_name, users.table_name);
    assert_ne!(blog.columns.len(), users.columns.len());
}
