use crate::migration::tests::*;

// ── Enum column default: emitted on the Enum coldef, all engines ──

// Regression: the generator dropped `[default: ...]` on enum columns (the `{default}`
// fragment was missing from the enum branch of render_column_def). A NOT NULL enum
// column added to a populated table then failed with "contains null values".

const ENUM_DEFAULT_MODEL: &str = r#"
use runique::prelude::*;
model! {
    Article,
    table: "article",
    pk: id => Pk,
    enums: {
        Status: [Draft="Draft", Published="Published", Archived="Archived"],
    },
    {
        status: choice [enum(Status), default: "Draft", required],
    }
}
"#;

#[test]
fn enum_default_reaches_parsed_column() {
    let schema = parse_model(ENUM_DEFAULT_MODEL);
    assert_eq!(
        col(&schema, "status").default_value.as_deref(),
        Some("\"Draft\""),
        "model! must capture `default` alongside `enum(...)`"
    );
}

#[test]
fn enum_column_default_is_emitted_in_create_on_all_engines() {
    let schema = parse_model(ENUM_DEFAULT_MODEL);
    let sql = generate_create_file(&schema);
    assert!(
        sql.contains("ColumnType::Enum"),
        "enum coldef missing:\n{sql}"
    );
    assert!(
        sql.contains(r#".not_null().default("Draft")"#),
        "enum default dropped:\n{sql}"
    );
}

const EXTEND_ENUM_DEFAULT_SRC: &str = r#"
use runique::prelude::*;
extend! {
    table: "eihwaz_users",
    enums: {
        Status: [Draft="Draft", Published="Published", Archived="Archived"],
    },
    fields: {
        status: choice [enum(Status), default: "Draft", required],
    }
}
"#;

#[test]
fn extend_enum_column_default_is_emitted_on_add_all_engines() {
    // The production scenario: ADD COLUMN <enum> NOT NULL on an existing, populated
    // table must carry a DEFAULT so Postgres can backfill instead of erroring.
    let schema = &parse_extend_blocks_from_source(EXTEND_ENUM_DEFAULT_SRC).unwrap()[0];
    let change = Changes {
        table_name: schema.table_name.clone(),
        added_columns: schema.columns.clone(),
        ..Default::default()
    };
    // A single generated file serves all engines.
    let sql = generate_alter_file(&change);
    assert!(
        sql.contains(r#".not_null().default("Draft")"#),
        "extend enum default missing on ADD COLUMN:\n{sql}"
    );
    assert!(
        sql.contains("get_database_backend() == sea_orm::DbBackend::Postgres"),
        "the type of the column must be chosen when the migration runs:\n{sql}"
    );
    // The type is the enum's to create, in its own migration, earlier in the
    // run: the file that adds the column finds it there.
    assert!(
        !sql.contains("Type::create()") && !sql.contains("create_enum_type"),
        "the file of the table must not create the enum type:\n{sql}"
    );
    let found = enums_of_schemas(std::slice::from_ref(schema));
    let created = generate_enum_file(
        &diff_enum(None, &found[0]),
        &crate::db::DatabaseEngine::PostgreSQL,
    );
    assert!(
        created.contains(
            r#"runique_cli::migration::runtime::create_enum_type(manager, "status", &["Draft", "Published", "Archived"]).await?;"#
        ),
        "the enum's own migration creates the type with the default among its values:\n{created}"
    );
}
