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
fn extend_enum_column_emits_create_type_on_postgres() {
    let schema = &parse_extend_blocks_from_source(EXTEND_ENUM_SRC).unwrap()[0];
    let change = Changes {
        table_name: schema.table_name.clone(),
        added_columns: schema.columns.clone(),
        ..Default::default()
    };
    let sql = generate_alter_file(&change);
    assert!(
        sql.contains("get_database_backend() == sea_orm::DbBackend::Postgres"),
        "must check the real backend at runtime:\n{sql}"
    );
    assert!(
        sql.contains("Type::create()"),
        "extend enum must create its type, runtime-guarded to Postgres:\n{sql}"
    );
}
