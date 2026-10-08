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
        {
            title: text [required],
            summary: text [nullable],
            views: int [required],
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
        {
            username: text [required, unique],
            email: email [required, unique],
            is_active: bool [nullable, default: true],
            created_at: datetime [auto_now],
            updated_at: datetime [auto_now_update],
        }
    }
    "#
}

// ── Parsing réussi ────────────────────────────────────────────────────────────

#[test]
fn test_parse_schema_returns_some() {
    let result = parse_schema_from_source(blog_source()).unwrap();
    assert!(
        result.is_some(),
        "Le parser doit retourner Some pour un model! valide"
    );
}

#[test]
fn test_parse_schema_table_name() {
    let schema = parse_schema_from_source(blog_source()).unwrap().unwrap().1;
    assert_eq!(schema.table_name, "blog");
}

#[test]
fn test_parse_schema_primary_key() {
    let schema = parse_schema_from_source(blog_source()).unwrap().unwrap().1;
    let pk = schema.primary_key.as_ref().unwrap();
    assert_eq!(pk.name, "id");
}

#[test]
fn test_parse_schema_field_count() {
    let schema = parse_schema_from_source(blog_source()).unwrap().unwrap().1;
    // 4 champs: title, summary, views, published
    assert_eq!(schema.columns.len(), 4);
}

#[test]
fn test_parse_schema_field_names() {
    let schema = parse_schema_from_source(blog_source()).unwrap().unwrap().1;
    let names: Vec<&str> = schema.columns.iter().map(|c| c.name.as_str()).collect();
    assert!(names.contains(&"title"));
    assert!(names.contains(&"summary"));
    assert!(names.contains(&"views"));
    assert!(names.contains(&"published"));
}

// ── Options des champs ────────────────────────────────────────────────────────

#[test]
fn test_parse_nullable_field() {
    let schema = parse_schema_from_source(blog_source()).unwrap().unwrap().1;
    let summary = schema.columns.iter().find(|c| c.name == "summary").unwrap();
    assert!(summary.nullable, "summary doit être nullable");
}

#[test]
fn test_parse_non_nullable_field() {
    let schema = parse_schema_from_source(blog_source()).unwrap().unwrap().1;
    let title = schema.columns.iter().find(|c| c.name == "title").unwrap();
    assert!(!title.nullable, "title ne doit pas être nullable");
}

#[test]
fn test_parse_unique_field() {
    let schema = parse_schema_from_source(users_source()).unwrap().unwrap().1;
    let username = schema
        .columns
        .iter()
        .find(|c| c.name == "username")
        .unwrap();
    assert!(username.unique, "username doit être unique");
}

#[test]
fn test_parse_auto_now_becomes_datetime_and_ignored() {
    let schema = parse_schema_from_source(users_source()).unwrap().unwrap().1;
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
    let result = parse_schema_from_source("").unwrap();
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
    let result = parse_schema_from_source(source).unwrap();
    assert!(result.is_none(), "Pas de macro model! → None");
}

// ── Enums dans le DSL ─────────────────────────────────────────────────────────

fn enum_string_source() -> &'static str {
    r#"
    use runique::prelude::*;
    model! {
        Article,
        table: "articles",
        pk: id => i32,
        enums: {
            Status: [Draft="Draft", Published="Published", Archived="Archive"],
        },
        {
            title: text [required],
            status: choice [enum(Status), required],
        }
    }
    "#
}

fn enum_i32_source() -> &'static str {
    r#"
    use runique::prelude::*;
    model! {
        Task,
        table: "tasks",
        pk: id => i32,
        enums: {
            Priority: i32 [Low=1, Medium=2, High=3],
        },
        {
            name: text [required],
            priority: choice [enum(Priority), required],
        }
    }
    "#
}

#[test]
fn test_parse_enum_string_schema_valide() {
    let result = parse_schema_from_source(enum_string_source()).unwrap();
    assert!(result.is_some(), "model! avec enum String doit parser");
}

#[test]
fn test_parse_enum_string_field_type_est_string() {
    let schema = parse_schema_from_source(enum_string_source())
        .unwrap()
        .unwrap()
        .1;
    let status = schema.columns.iter().find(|c| c.name == "status").unwrap();
    assert_eq!(status.col_type, "String", "enum String → col_type String");
}

#[test]
fn test_parse_enum_string_values_sont_collectes() {
    let schema = parse_schema_from_source(enum_string_source())
        .unwrap()
        .unwrap()
        .1;
    let status = schema.columns.iter().find(|c| c.name == "status").unwrap();
    assert_eq!(status.enum_string_values.len(), 3);
    assert!(status.enum_string_values.contains(&"Draft".to_string()));
    assert!(status.enum_string_values.contains(&"Published".to_string()));
    assert!(status.enum_string_values.contains(&"Archive".to_string()));
}

#[test]
fn test_parse_enum_string_values_valeur_explicite() {
    let schema = parse_schema_from_source(enum_string_source())
        .unwrap()
        .unwrap()
        .1;
    let status = schema.columns.iter().find(|c| c.name == "status").unwrap();
    // "Archived" est le nom du variant mais "Archive" est la valeur DB
    assert!(
        status.enum_string_values.contains(&"Archive".to_string()),
        "La valeur DB explicite 'Archive' doit être stockée, pas le nom variant"
    );
}

#[test]
fn test_parse_enum_string_enum_name_stocke() {
    let schema = parse_schema_from_source(enum_string_source())
        .unwrap()
        .unwrap()
        .1;
    let status = schema.columns.iter().find(|c| c.name == "status").unwrap();
    assert_eq!(status.enum_name.as_deref(), Some("Status"));
}

#[test]
fn test_parse_enum_i32_field_type_est_integer() {
    let schema = parse_schema_from_source(enum_i32_source())
        .unwrap()
        .unwrap()
        .1;
    let priority = schema
        .columns
        .iter()
        .find(|c| c.name == "priority")
        .unwrap();
    assert_eq!(priority.col_type, "Integer", "enum i32 → col_type Integer");
}

#[test]
fn test_parse_enum_i32_pas_de_string_values() {
    // Les enums i32 n'ont pas de string_values dans le snapshot
    let schema = parse_schema_from_source(enum_i32_source())
        .unwrap()
        .unwrap()
        .1;
    let priority = schema
        .columns
        .iter()
        .find(|c| c.name == "priority")
        .unwrap();
    assert!(
        priority.enum_string_values.is_empty(),
        "enum i32 ne doit pas stocker de string_values"
    );
}

#[test]
fn test_parse_non_enum_field_pas_de_string_values() {
    let schema = parse_schema_from_source(enum_string_source())
        .unwrap()
        .unwrap()
        .1;
    let title = schema.columns.iter().find(|c| c.name == "title").unwrap();
    assert!(title.enum_string_values.is_empty());
    assert!(title.enum_name.is_none());
}

// ── `Pk` sur un champ normal (FK) ──────────────────────────────────────────────

fn contributions_source() -> &'static str {
    r#"
    use runique::prelude::*;
    model! {
        Contributions,
        table: "contributions",
        pk: id => Pk,
        {
            user_id: Pk [required],
            title: text [nullable],
        }
    }
    "#
}

// Bug (2026-09-01) : `dsl_field_type_to_col_type` (champs normaux) n'avait pas
// de branche pour "Pk" — seule `dsl_pk_to_col_type` (position `pk:`) la
// gérait. Un FK typé `Pk` (convention documentée pour rester en phase avec
// big-pk/pk-uuid) tombait silencieusement dans le `_ => "String"`, détecté en
// vrai sur `contributions.user_id`/`chapitre.cour_id`/etc. via
// `runique makemigrations` sur demo-app (faux changements destructifs
// Integer -> String). `Pk` change de type selon la feature active
// (i32/i64/Uuid, cf. `test_eihwaz_tables_pk.rs`), donc l'assertion doit
// suivre la même matrice plutôt que supposer les features par défaut — un
// job CI `--features big-pk` a cassé la version non-gated de ce test.

#[cfg(not(any(feature = "big-pk", feature = "pk-uuid")))]
#[test]
fn test_parse_pk_typed_field_maps_to_integer() {
    let schema = parse_schema_from_source(contributions_source())
        .unwrap()
        .unwrap()
        .1;
    let user_id = schema.columns.iter().find(|c| c.name == "user_id").unwrap();
    assert_eq!(
        user_id.col_type, "Integer",
        "un champ Pk (features par défaut) doit être Integer"
    );
}

#[cfg(all(feature = "big-pk", not(feature = "pk-uuid")))]
#[test]
fn test_parse_pk_typed_field_maps_to_big_integer() {
    let schema = parse_schema_from_source(contributions_source())
        .unwrap()
        .unwrap()
        .1;
    let user_id = schema.columns.iter().find(|c| c.name == "user_id").unwrap();
    assert_eq!(
        user_id.col_type, "BigInteger",
        "un champ Pk (feature big-pk) doit être BigInteger"
    );
}

#[cfg(feature = "pk-uuid")]
#[test]
fn test_parse_pk_typed_field_maps_to_uuid() {
    let schema = parse_schema_from_source(contributions_source())
        .unwrap()
        .unwrap()
        .1;
    let user_id = schema.columns.iter().find(|c| c.name == "user_id").unwrap();
    assert_eq!(
        user_id.col_type, "Uuid",
        "un champ Pk (feature pk-uuid) doit être Uuid"
    );
}

// ── Isolation entre tables ────────────────────────────────────────────────────

#[test]
fn test_parse_different_tables_independent() {
    let blog = parse_schema_from_source(blog_source()).unwrap().unwrap().1;
    let users = parse_schema_from_source(users_source()).unwrap().unwrap().1;
    assert_ne!(blog.table_name, users.table_name);
    assert_ne!(blog.columns.len(), users.columns.len());
}

/// `auto_now` / `auto_now_update` make a date-time column, except that a
/// `timestamp_tz` keeps its time zone — in the CLI's schema and in the
/// model's own. Its field is a `DateTime<Utc>`: on Postgres, a plain
/// `TIMESTAMP` column can't be read into it.
#[test]
fn auto_now_keeps_the_time_zone_of_a_timestamp_tz() {
    let src = r#"model! { Stamp, table: "stamps", pk: id => i32, {
        created: timestamp_tz [auto_now],
        updated: timestamp_tz [auto_now_update],
        seen: datetime [auto_now],
        logged: timestamp [auto_now],
    } }"#;
    let schema = parse_schema_from_source(src).unwrap().unwrap().1;
    let ty = |name: &str| {
        let c = schema.columns.iter().find(|c| c.name == name).unwrap();
        assert!(c.has_default_now, "{name}");
        c.col_type.clone()
    };
    assert_eq!(ty("created"), "TimestampWithTimeZone");
    assert_eq!(ty("updated"), "TimestampWithTimeZone");
    assert_eq!(ty("seen"), "DateTime");
    assert_eq!(ty("logged"), "DateTime");

    use runique::migration::column::ColumnDef;
    use runique::sea_orm::sea_query::ColumnType;
    let tz = ColumnDef::new("c").timestamp_tz().auto_now();
    assert!(matches!(tz.col_type, ColumnType::TimestampWithTimeZone));
    let tz = ColumnDef::new("c").timestamp_tz().auto_now_update();
    assert!(matches!(tz.col_type, ColumnType::TimestampWithTimeZone));
    let plain = ColumnDef::new("c").timestamp().auto_now();
    assert!(matches!(plain.col_type, ColumnType::DateTime));
    let bare = ColumnDef::new("c").auto_now_update();
    assert!(
        matches!(bare.col_type, ColumnType::DateTime),
        "no type given: a date-time"
    );
}
