use crate::migration::tests::*;

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
    let changes = diff_schemas(&parse_model(ENUM_V1), &parse_model(ENUM_V2));
    assert!(
        !changes.enum_value_adds.is_empty(),
        "adding an enum variant should be detected: {changes:?}"
    );
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
    let changes = diff_schemas(&parse_model(ENUM_V1), &parse_model(ENUM_RENAMED));
    assert_eq!(
        changes.enum_renames.len(),
        1,
        "exactly one rename: {changes:?}"
    );
    let (col, enum_name, old, new) = &changes.enum_renames[0];
    assert_eq!(
        (col.as_str(), enum_name.as_str(), old.as_str(), new.as_str()),
        ("status", "Status", "Published", "Release")
    );
    assert!(
        changes.enum_value_adds.is_empty(),
        "rename must not appear as an add"
    );
    assert!(
        changes.enum_value_drops.is_empty(),
        "rename must not appear as a drop"
    );
}

#[test]
fn enum_rename_branches_on_runtime_backend() {
    // Which branch runs is decided by `manager.get_database_backend()` at
    // migration-run time, not at generation time: a single generated file must
    // carry BOTH branches.
    let changes = diff_schemas(&parse_model(ENUM_V1), &parse_model(ENUM_RENAMED));
    let sql = generate_alter_file(&changes);
    assert!(
        sql.contains("get_database_backend() == sea_orm::DbBackend::Postgres"),
        "must check the real backend at runtime:\n{sql}"
    );
    // Postgres branch: `sea_query::extension::postgres::Type::alter()` builder
    // (escapes values automatically), not a hand-written `ALTER TYPE` string.
    assert!(
        sql.contains(".name(Alias::new(\"status\"))")
            && sql.contains(".rename_value(Alias::new(\"Published\"), Alias::new(\"Release\"))"),
        "Postgres branch must use the Type::alter().rename_value() builder, with the \
         type name lowercased to match the unquoted CREATE TYPE's case-folded identity:\n{sql}"
    );
    // non-Postgres branch: `Query::update()` builder, same reasoning.
    assert!(
        sql.contains(".table(Alias::new(\"post\"))")
            && sql.contains(".value(Alias::new(\"status\"), \"Release\")")
            && sql.contains(".eq(\"Published\")"),
        "non-Postgres branch must use the Query::update() builder:\n{sql}"
    );
}

#[test]
fn enum_create_type_is_postgres_only() {
    // `CREATE TYPE … AS ENUM` is Postgres-only syntax: it's always emitted,
    // wrapped in a runtime backend check, so the same generated file works no
    // matter which engine actually runs it later.
    let schema = parse_model(ENUM_V2);
    let sql = generate_create_file(&schema);
    assert!(
        sql.contains("get_database_backend() == sea_orm::DbBackend::Postgres"),
        "must check the real backend at runtime:\n{sql}"
    );
    assert!(
        sql.contains("Type::create()") && sql.contains(".as_enum(Alias::new(\"status\"))"),
        "the enum type must always be created (runtime-guarded):\n{sql}"
    );
    assert!(
        sql.contains(
            r#"runique_cli::migration::runtime::drop_enum_type(manager, "status").await?;"#
        ),
        "down() must always hand the type over to be dropped once unused:\n{sql}"
    );
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
