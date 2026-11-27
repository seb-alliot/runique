use crate::db::DatabaseEngine;
use crate::migration::tests::*;

/// The enum of a model, as `makemigrations` finds it.
fn status(src: &str) -> ParsedEnum {
    enums_of_schemas(&[parse_model(src)]).remove(0)
}

// ── Enum value addition ──

const ENUM_V1: &str = r#"
use runique::prelude::*;
model! {
    Post,
    table: "post",
    pk: id => Pk,
    enums: {
        Status: [Draft="Draft", Published="Published"],
    },
    {
        status: choice [enum(Status), required],
    }
}
"#;

const ENUM_V2: &str = r#"
use runique::prelude::*;
model! {
    Post,
    table: "post",
    pk: id => Pk,
    enums: {
        Status: [Draft="Draft", Published="Published", Archived="Archived"],
    },
    {
        status: choice [enum(Status), required],
    }
}
"#;

const ENUM_V1_NO_STATUS: &str = r#"
use runique::prelude::*;
model! {
    Post,
    table: "post",
    pk: id => Pk,
    {
        title: text [required],
    }
}
"#;

#[test]
fn enum_value_addition_is_detected() {
    let changes = diff_enum(Some(&status(ENUM_V1)), &status(ENUM_V2));
    assert_eq!(changes.added_values, ["Archived"]);
    // The table itself has nothing to migrate: the value is the enum's.
    assert!(diff_schemas(&parse_model(ENUM_V1), &parse_model(ENUM_V2)).is_empty());
}

const ENUM_RENAMED: &str = r#"
use runique::prelude::*;
model! {
    Post,
    table: "post",
    pk: id => Pk,
    enums: {
        Status: [Draft="Draft", Published="Release"],
    },
    {
        status: choice [enum(Status), required],
    }
}
"#;

#[test]
fn enum_rename_is_a_single_operation() {
    // A positional value change is ONE rename — it must not leak into add/drop.
    let changes = diff_enum(Some(&status(ENUM_V1)), &status(ENUM_RENAMED));
    assert_eq!(
        changes.renamed_values,
        [("Published".to_string(), "Release".to_string())]
    );
    assert!(
        changes.added_values.is_empty(),
        "rename must not appear as an add"
    );
    assert!(
        changes.dropped_values.is_empty(),
        "rename must not appear as a drop"
    );
    assert_eq!(
        changes.places,
        [EnumPlace {
            table: "post".into(),
            column: "status".into()
        }]
    );
}

#[test]
fn enum_rename_is_handed_to_the_runtime_with_its_places() {
    // The file names the operation; which engine runs it, and so whether the
    // type or the rows are written, is read when the migration runs.
    let changes = diff_enum(Some(&status(ENUM_V1)), &status(ENUM_RENAMED));
    for engine in [
        DatabaseEngine::PostgreSQL,
        DatabaseEngine::MySQL,
        DatabaseEngine::SQLite,
    ] {
        let sql = generate_enum_file(&changes, &engine);
        assert!(
            sql.contains(
                r#"runique_cli::migration::runtime::rename_enum_value(manager, "status", "Published", "Release", &[("post", "status")]).await?;"#
            ),
            "{sql}"
        );
        assert!(!sql.contains("get_database_backend()"), "{sql}");
    }
    // The table's own file holds nothing about it.
    assert!(diff_schemas(&parse_model(ENUM_V1), &parse_model(ENUM_RENAMED)).is_empty());
}

#[test]
fn enum_create_type_is_in_the_file_of_the_enum_on_postgres_only() {
    // The type is the enum's to create, once, before the tables: the file
    // that creates a table holds neither its creation nor its drop.
    let schema = parse_model(ENUM_V2);
    let table = generate_create_file(&schema);
    assert!(
        !table.contains("create_enum_type") && !table.contains("Type::create()"),
        "{table}"
    );
    assert!(!table.contains("drop_enum_type"), "{table}");

    let created = diff_enum(None, &status(ENUM_V2));
    let on_postgres = generate_enum_file(&created, &DatabaseEngine::PostgreSQL);
    assert!(
        on_postgres.contains(
            r#"runique_cli::migration::runtime::create_enum_type(manager, "status", &["Draft", "Published", "Archived"]).await?;"#
        ),
        "{on_postgres}"
    );
    assert!(
        on_postgres.contains(
            r#"runique_cli::migration::runtime::drop_enum_type(manager, "status").await?;"#
        ),
        "down() hands the type over to be dropped once unused:\n{on_postgres}"
    );
    for engine in [DatabaseEngine::MySQL, DatabaseEngine::SQLite] {
        let elsewhere = generate_enum_file(&created, &engine);
        assert!(!elsewhere.contains("create_enum_type"), "{elsewhere}");
        assert!(!elsewhere.contains("drop_enum_type"), "{elsewhere}");
    }
}

#[test]
fn enum_column_is_a_native_enum_on_postgres_only() {
    // sea-query renders `ColumnType::Enum` as a native `ENUM(...)` on MySQL /
    // MySQL, whose value list no later migration updates: off Postgres the
    // column must be a plain string, like the entity reads it.
    let sql = generate_create_file(&parse_model(ENUM_V2));
    let guarded = "new_with_type(Alias::new(\"status\"), if manager.get_connection().get_database_backend() == sea_orm::DbBackend::Postgres { ColumnType::Enum { name: Alias::new(\"Status\").into_iden()";
    assert!(
        sql.contains(guarded),
        "the enum type must be chosen at run time:\n{sql}"
    );
    assert!(
        sql.contains("} else { ColumnType::String(Default::default()) })"),
        "off Postgres, the column must be a plain string:\n{sql}"
    );
    assert!(
        !sql.contains("new_with_type(Alias::new(\"status\"), ColumnType::Enum"),
        "no unguarded native enum left:\n{sql}"
    );
}

#[test]
fn an_added_enum_column_is_a_native_enum_on_postgres_only() {
    let changes = diff_schemas(&parse_model(ENUM_V1_NO_STATUS), &parse_model(ENUM_V2));
    let sql = generate_alter_file(&changes);
    assert!(
        sql.contains("} else { ColumnType::String(Default::default()) })"),
        "the added column must be a plain string off Postgres:\n{sql}"
    );
    assert!(
        !sql.contains("new_with_type(Alias::new(\"status\"), ColumnType::Enum"),
        "no unguarded native enum left:\n{sql}"
    );
}

#[test]
fn guarded_enum_snapshot_round_trip_is_stable() {
    let schema = parse_model(ENUM_V2);
    let snapshot = generate_snapshot_file(&schema);
    let reparsed = parse_seaorm_source(&snapshot).expect("snapshot should reparse");
    assert_eq!(
        col(&reparsed, "status").enum_string_values,
        ["Draft", "Published", "Archived"]
    );
    let changes = diff_schemas(&reparsed, &schema);
    assert!(
        changes.is_empty(),
        "round-trip not stable, spurious changes: {changes:?}"
    );
}

#[test]
fn an_unguarded_enum_column_of_an_older_migration_still_reads() {
    // Migrations written before the run-time choice keep their bare
    // `ColumnType::Enum`: read back, they must not look like a change.
    let old = r#"
use sea_orm_migration::prelude::*;
#[derive(DeriveMigrationName)]
pub struct Migration;
#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Alias::new("post"))
                    .if_not_exists()
                    .col(ColumnDef::new(Alias::new("id")).integer().not_null().auto_increment().primary_key())
                    .col(ColumnDef::new_with_type(Alias::new("status"), ColumnType::Enum { name: Alias::new("Status").into_iden(), variants: vec![Alias::new("Draft").into_iden(), Alias::new("Published").into_iden(), Alias::new("Archived").into_iden()] }).not_null())
                    .to_owned(),
            )
            .await?;
        Ok(())
    }
    async fn down(&self, _manager: &SchemaManager) -> Result<(), DbErr> {
        Ok(())
    }
}
"#;
    let parsed = parse_seaorm_source(old).expect("older migration should parse");
    assert_eq!(
        col(&parsed, "status").enum_string_values,
        ["Draft", "Published", "Archived"]
    );
    assert_eq!(col(&parsed, "status").enum_name.as_deref(), Some("Status"));
}
