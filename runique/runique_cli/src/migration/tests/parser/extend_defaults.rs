use crate::migration::tests::*;

// ── extend!{} default capture ──

const EXTEND_SRC: &str = r#"
use runique::prelude::*;
extend! {
    table: "eihwaz_users",
    fields: {
        is_verified: bool [nullable, default: false],
        job_title:   text [nullable],
    }
}
"#;

#[test]
fn extend_captures_literal_default() {
    let schemas = parse_extend_blocks_from_source(EXTEND_SRC).unwrap();
    let schema = schemas.first().expect("one extend block");
    assert_eq!(
        col(schema, "is_verified").default_value.as_deref(),
        Some("false"),
        "extend!{{}} must capture literal defaults"
    );
}

const EXTEND_ENUM_SRC: &str = r#"
use runique::prelude::*;
extend! {
    table: "eihwaz_users",
    enums: {
        Role: [Admin="admin", Staff="staff", Guest="guest"],
    },
    fields: {
        role: choice [enum(Role), required],
    }
}
"#;

#[test]
fn extend_resolves_enum_column() {
    let schemas = parse_extend_blocks_from_source(EXTEND_ENUM_SRC).unwrap();
    let schema = schemas.first().expect("one extend block");
    let role = col(schema, "role");
    assert_eq!(role.enum_name.as_deref(), Some("Role"));
    assert_eq!(
        role.enum_string_values,
        vec!["admin", "staff", "guest"],
        "extend!{{}} must resolve enum variants for a choice column"
    );
    assert_eq!(
        role.col_type, "String",
        "Auto-backed enum is a VARCHAR/enum column"
    );
    assert!(!role.nullable, "required choice must be NOT NULL");
}

#[test]
fn extend_enum_is_created_by_its_own_migration_on_postgres() {
    let schema = &parse_extend_blocks_from_source(EXTEND_ENUM_SRC).unwrap()[0];
    let change = Changes {
        table_name: schema.table_name.clone(),
        added_columns: schema.columns.clone(),
        ..Default::default()
    };
    // The file that adds the column finds the enum there.
    let sql = generate_alter_file(&change);
    assert!(
        !sql.contains("create_enum_type") && !sql.contains("Type::create()"),
        "{sql}"
    );

    // The enum of an `extend!{}` block is found like the one of a model, and
    // has its own file.
    let found = enums_of_schemas(std::slice::from_ref(schema));
    assert_eq!(found.len(), 1, "{found:?}");
    assert_eq!(found[0].name, "role");
    let created = generate_enum_file(
        &diff_enum(None, &found[0]),
        &crate::db::DatabaseEngine::PostgreSQL,
    );
    assert!(
        created
            .contains(r#"runique_cli::migration::runtime::create_enum_type(manager, "role", &["#),
        "{created}"
    );
}
