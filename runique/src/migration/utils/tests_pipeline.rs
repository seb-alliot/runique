//! Pure pipeline tests for `makemigrations` — no DB, no Docker.
//!
//! These cover the regressions found during the 2026-06-11 session:
//! - literal `[default: X]` values must be parsed and emitted,
//! - `bool` is a v2 type (nullable unless `required`),
//! - a CASCADE FK on a brand-new table is not a destructive change,
//! - the snapshot round-trip is stable (parse → snapshot → reparse → diff empty),
//! - enum value additions are detected.
#![cfg(test)]
use super::*;
use crate::utils::cli::makemigration::{collect_destructive_messages, topological_sort_changes};

// ── Helpers ──────────────────────────────────────────────────────────────────

fn parse_model(src: &str) -> ParsedSchema {
    parse_schema_from_source(src)
        .expect("model! source should parse")
        .1
}

fn col<'a>(schema: &'a ParsedSchema, name: &str) -> &'a ParsedColumn {
    schema
        .columns
        .iter()
        .find(|c| c.name == name)
        .unwrap_or_else(|| panic!("column `{name}` not found"))
}

/// Minimal `Changes` for destructive-guard tests.
fn change(
    is_new_table: bool,
    added_fks: Vec<ParsedFk>,
    dropped_columns: Vec<ParsedColumn>,
) -> Changes {
    Changes {
        table_name: "t".into(),
        added_columns: vec![],
        dropped_columns,
        modified_columns: vec![],
        added_fks,
        dropped_fks: vec![],
        added_indexes: vec![],
        dropped_indexes: vec![],
        is_new_table,
        renamed_columns: vec![],
        enum_renames: vec![],
        enum_value_adds: vec![],
        enum_value_drops: vec![],
    }
}

fn cascade_fk() -> ParsedFk {
    ParsedFk {
        from_column: "parent_id".into(),
        to_table: "parent".into(),
        to_column: "id".into(),
        on_delete: "CASCADE".into(),
        on_update: "NoAction".into(),
    }
}

const BLOG_SRC: &str = r#"
use runique::prelude::*;
model! {
    Blog,
    table: "blog",
    pk: id => Pk,
    {
        title:      text [required],
        view_count: int  [default: 0],
        is_active:  bool [default: true],
    }
}
"#;

// ── Literal defaults ─────────────────────────────────────────────────────────

#[test]
fn literal_default_is_parsed() {
    let schema = parse_model(BLOG_SRC);
    assert_eq!(
        col(&schema, "view_count").default_value.as_deref(),
        Some("0")
    );
    assert_eq!(
        col(&schema, "is_active").default_value.as_deref(),
        Some("true")
    );
}

#[test]
fn literal_default_is_emitted_in_create() {
    let sql = generate_create_file(&parse_model(BLOG_SRC), &DbKind::Postgres);
    assert!(sql.contains(".default(0)"), "int default missing:\n{sql}");
    assert!(
        sql.contains(".default(true)"),
        "bool default missing:\n{sql}"
    );
}

#[test]
fn bool_without_required_is_nullable() {
    // `bool` is now a v2 type → nullable unless `required`.
    assert!(
        col(&parse_model(BLOG_SRC), "is_active").nullable,
        "bool without `required` should be nullable"
    );
}

// ── Snapshot round-trip (the golden stability test) ────────────────────────────

#[test]
fn snapshot_default_does_not_set_has_default_now() {
    // parser_seaorm must NOT treat a literal `.default(0)` as a CURRENT_TIMESTAMP default.
    let schema = parse_model(BLOG_SRC);
    let snapshot = generate_snapshot_file(&schema);
    let reparsed = parse_seaorm_source(&snapshot).expect("snapshot should reparse");
    assert!(
        !col(&reparsed, "view_count").has_default_now,
        "literal default must not be read back as a timestamp default"
    );
}

#[test]
fn snapshot_round_trip_is_stable() {
    // The invariant behind "makemigrations twice = no changes".
    let schema = parse_model(BLOG_SRC);
    let snapshot = generate_snapshot_file(&schema);
    let reparsed = parse_seaorm_source(&snapshot).expect("snapshot should reparse");
    let changes = diff_schemas(&reparsed, &schema);
    assert!(
        changes.is_empty(),
        "round-trip not stable, spurious changes: {changes:?}"
    );
}

// ── Destructive guard ──────────────────────────────────────────────────────────

#[test]
fn cascade_fk_on_new_table_is_not_destructive() {
    let changes = vec![change(true, vec![cascade_fk()], vec![])];
    assert!(
        collect_destructive_messages(&changes).is_empty(),
        "CASCADE FK on a brand-new table must not be flagged destructive"
    );
}

#[test]
fn cascade_fk_on_existing_table_is_destructive() {
    let changes = vec![change(false, vec![cascade_fk()], vec![])];
    let msgs = collect_destructive_messages(&changes);
    assert!(
        msgs.iter().any(|m| m.contains("CASCADE")),
        "CASCADE FK on an existing table must be flagged: {msgs:?}"
    );
}

#[test]
fn drop_column_is_destructive() {
    let dropped = vec![ParsedColumn {
        name: "removed".into(),
        ..Default::default()
    }];
    let changes = vec![change(false, vec![], dropped)];
    let msgs = collect_destructive_messages(&changes);
    assert!(
        msgs.iter().any(|m| m.contains("DROP COLUMN")),
        "dropping a column must be flagged: {msgs:?}"
    );
}

// ── Enum value addition ────────────────────────────────────────────────────────

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
fn enum_rename_uses_alter_type_rename_on_postgres() {
    let changes = diff_schemas(&parse_model(ENUM_V1), &parse_model(ENUM_RENAMED));
    let pg = generate_alter_file(&changes, &DbKind::Postgres);
    assert!(
        pg.contains("ALTER TYPE Status RENAME VALUE 'Published' TO 'Release'"),
        "PG rename must use ALTER TYPE RENAME VALUE:\n{pg}"
    );
    assert!(
        !pg.contains("UPDATE"),
        "PG rename must not UPDATE rows:\n{pg}"
    );
    assert!(
        !pg.contains("ADD VALUE"),
        "PG rename must not ADD/DROP the value:\n{pg}"
    );
}

#[test]
fn enum_rename_updates_data_on_non_postgres() {
    let changes = diff_schemas(&parse_model(ENUM_V1), &parse_model(ENUM_RENAMED));
    let other = generate_alter_file(&changes, &DbKind::Other);
    assert!(
        other.contains("UPDATE post SET status = 'Release' WHERE status = 'Published'"),
        "non-PG rename updates data:\n{other}"
    );
    assert!(
        !other.contains("ALTER TYPE"),
        "non-PG must not emit ALTER TYPE:\n{other}"
    );
}

#[test]
fn enum_create_type_is_postgres_only() {
    // `CREATE TYPE … AS ENUM` is a PostgreSQL-only construct, gated by DbKind
    // (resolved from DB_ENGINE / DB_URL at runtime). Other backends render the
    // enum inline without a separate type.
    let schema = parse_model(ENUM_V2);
    let pg = generate_create_file(&schema, &DbKind::Postgres);
    assert!(
        pg.contains("CREATE TYPE"),
        "Postgres must emit CREATE TYPE:\n{pg}"
    );
    assert!(
        pg.contains("DROP TYPE"),
        "Postgres down() must drop the type:\n{pg}"
    );

    let other = generate_create_file(&schema, &DbKind::Other);
    assert!(
        !other.contains("CREATE TYPE"),
        "non-Postgres must NOT emit CREATE TYPE:\n{other}"
    );

    let mysql = generate_create_file(&schema, &DbKind::Mysql);
    assert!(
        !mysql.contains("CREATE TYPE"),
        "MySQL must NOT emit CREATE TYPE:\n{mysql}"
    );
}

// ── extend!{} default capture ──────────────────────────────────────────────────

const EXTEND_SRC: &str = r#"
use runique::prelude::*;
extend! {
    table: "eihwaz_users",
    fields: {
        is_verified: bool [default: false],
        job_title:   text,
    }
}
"#;

#[test]
fn extend_captures_literal_default() {
    let schemas = parse_extend_blocks_from_source(EXTEND_SRC);
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
    let schemas = parse_extend_blocks_from_source(EXTEND_ENUM_SRC);
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
    let schema = &parse_extend_blocks_from_source(EXTEND_ENUM_SRC)[0];
    let change = Changes {
        table_name: schema.table_name.clone(),
        added_columns: schema.columns.clone(),
        dropped_columns: vec![],
        modified_columns: vec![],
        added_fks: vec![],
        dropped_fks: vec![],
        added_indexes: vec![],
        dropped_indexes: vec![],
        is_new_table: false,
        renamed_columns: vec![],
        enum_renames: vec![],
        enum_value_adds: vec![],
        enum_value_drops: vec![],
    };
    let pg = generate_alter_file(&change, &DbKind::Postgres);
    assert!(
        pg.contains("CREATE TYPE"),
        "PG extend enum must CREATE TYPE:\n{pg}"
    );

    let other = generate_alter_file(&change, &DbKind::Other);
    assert!(
        !other.contains("CREATE TYPE"),
        "non-PG extend enum must not CREATE TYPE:\n{other}"
    );
}

// ── Column rename via `[renamed_from: "old"]` ──────────────────────────────────

const COL_BEFORE: &str = r#"
use runique::prelude::*;
model! {
    Person,
    table: "person",
    pk: id => Pk,
    {
        job_title: text,
        age: int,
    }
}
"#;

const COL_RENAMED: &str = r#"
use runique::prelude::*;
model! {
    Person,
    table: "person",
    pk: id => Pk,
    {
        title: text [renamed_from: "job_title"],
        age: int,
    }
}
"#;

const COL_NO_HINT: &str = r#"
use runique::prelude::*;
model! {
    Person,
    table: "person",
    pk: id => Pk,
    {
        title: text,
        age: int,
    }
}
"#;

#[test]
fn column_rename_is_detected_via_hint() {
    let changes = diff_schemas(&parse_model(COL_BEFORE), &parse_model(COL_RENAMED));
    assert_eq!(
        changes.renamed_columns,
        vec![("job_title".to_string(), "title".to_string())]
    );
    assert!(
        changes.added_columns.is_empty(),
        "rename must not add: {:?}",
        changes.added_columns
    );
    assert!(
        changes.dropped_columns.is_empty(),
        "rename must not drop: {:?}",
        changes.dropped_columns
    );
}

#[test]
fn column_rename_without_hint_is_drop_add() {
    // No `renamed_from` → the diff cannot guess intent: it's a destructive drop + add.
    let changes = diff_schemas(&parse_model(COL_BEFORE), &parse_model(COL_NO_HINT));
    assert!(changes.renamed_columns.is_empty());
    assert!(changes.added_columns.iter().any(|c| c.name == "title"));
    assert!(
        changes
            .dropped_columns
            .iter()
            .any(|c| c.name == "job_title")
    );
}

#[test]
fn column_rename_sql_is_portable_across_engines() {
    // RENAME COLUMN is supported by PG / MySQL+MariaDB / SQLite → same builder for all.
    let changes = diff_schemas(&parse_model(COL_BEFORE), &parse_model(COL_RENAMED));
    for kind in [DbKind::Postgres, DbKind::Mysql, DbKind::Other] {
        let sql = generate_alter_file(&changes, &kind);
        assert!(
            sql.contains(r#".rename_column(Alias::new("job_title"), Alias::new("title"))"#),
            "RENAME COLUMN missing for {kind:?}:\n{sql}"
        );
        assert!(
            !sql.contains("drop_column"),
            "rename must not DROP for {kind:?}:\n{sql}"
        );
        assert!(
            !sql.contains("add_column"),
            "rename must not ADD for {kind:?}:\n{sql}"
        );
    }
}

#[test]
fn column_rename_down_reverses() {
    let changes = diff_schemas(&parse_model(COL_BEFORE), &parse_model(COL_RENAMED));
    let sql = generate_alter_file(&changes, &DbKind::Postgres);
    let down = sql.split("async fn down").nth(1).unwrap_or("");
    assert!(
        down.contains(r#".rename_column(Alias::new("title"), Alias::new("job_title"))"#),
        "down must reverse the rename:\n{sql}"
    );
}

#[test]
fn stale_renamed_from_falls_back_to_add() {
    // If the old column still exists (stale hint), do NOT rename — treat `title` as a new column.
    let changes = diff_schemas(&parse_model(COL_NO_HINT), &parse_model(COL_RENAMED));
    // prev has `title`+`age`, curr has `title`(renamed_from job_title)+`age`.
    // job_title is absent from prev → guard fails → no rename.
    assert!(
        changes.renamed_columns.is_empty(),
        "stale hint must not rename: {:?}",
        changes.renamed_columns
    );
}

const EXTEND_COL_BEFORE: &str = r#"
use runique::prelude::*;
extend! {
    table: "eihwaz_users",
    fields: {
        job_title: text,
    }
}
"#;

const EXTEND_COL_RENAMED: &str = r#"
use runique::prelude::*;
extend! {
    table: "eihwaz_users",
    fields: {
        title: text [renamed_from: "job_title"],
    }
}
"#;

#[test]
fn extend_column_rename_is_detected() {
    let prev = parse_extend_blocks_from_source(EXTEND_COL_BEFORE).remove(0);
    let curr = parse_extend_blocks_from_source(EXTEND_COL_RENAMED).remove(0);
    let changes = diff_schemas(&prev, &curr);
    assert_eq!(
        changes.renamed_columns,
        vec![("job_title".to_string(), "title".to_string())]
    );
    assert!(changes.added_columns.is_empty());
    assert!(changes.dropped_columns.is_empty());
    let sql = generate_alter_file(&changes, &DbKind::Mysql);
    assert!(
        sql.contains(r#".rename_column(Alias::new("job_title"), Alias::new("title"))"#),
        "extend rename must emit RENAME COLUMN:\n{sql}"
    );
}

// ── CREATE TABLE: primary key types ───────────────────────────────────────────

fn model_pk(pk: &str) -> String {
    format!(
        "use runique::prelude::*;\nmodel! {{\n  Thing,\n  table: \"thing\",\n  pk: id => {pk},\n  {{\n    label: text,\n  }}\n}}\n"
    )
}

#[test]
fn pk_i32_is_autoincrement_integer() {
    let sql = generate_create_file(&parse_model(&model_pk("Pk")), &DbKind::Postgres);
    assert!(
        sql.contains(
            r#".col(ColumnDef::new(Alias::new("id")).integer().not_null().auto_increment().primary_key())"#
        ),
        "{sql}"
    );
}

#[test]
fn pk_i64_is_big_integer_autoincrement() {
    let sql = generate_create_file(&parse_model(&model_pk("i64")), &DbKind::Postgres);
    assert!(
        sql.contains(".big_integer().not_null().auto_increment().primary_key()"),
        "{sql}"
    );
}

#[test]
fn pk_uuid_has_no_autoincrement() {
    let sql = generate_create_file(&parse_model(&model_pk("uuid")), &DbKind::Postgres);
    assert!(sql.contains(".uuid().not_null().primary_key()"), "{sql}");
    assert!(
        !sql.contains("auto_increment"),
        "uuid PK must not auto_increment:\n{sql}"
    );
}

// ── CREATE TABLE: timestamps per engine ───────────────────────────────────────

const TS_MODEL: &str = r#"
use runique::prelude::*;
model! {
    Event,
    table: "event",
    pk: id => Pk,
    {
        created_at: datetime,
        updated_at: datetime,
    }
}
"#;

#[test]
fn created_at_defaults_to_current_timestamp() {
    let sql = generate_create_file(&parse_model(TS_MODEL), &DbKind::Other);
    assert!(sql.contains(".default(Expr::current_timestamp())"), "{sql}");
}

#[test]
fn updated_at_uses_on_update_extra_on_mysql() {
    let sql = generate_create_file(&parse_model(TS_MODEL), &DbKind::Mysql);
    assert!(
        sql.contains(r#".extra("ON UPDATE CURRENT_TIMESTAMP")"#),
        "MySQL updated_at must use ON UPDATE extra:\n{sql}"
    );
    assert!(
        !sql.contains("CREATE TRIGGER"),
        "MySQL must not use a trigger:\n{sql}"
    );
}

#[test]
fn updated_at_uses_trigger_on_postgres() {
    let sql = generate_create_file(&parse_model(TS_MODEL), &DbKind::Postgres);
    assert!(
        sql.contains("CREATE TRIGGER trg_event_updated_at"),
        "PG trigger:\n{sql}"
    );
    assert!(
        sql.contains("set_updated_at_event"),
        "PG trigger fn:\n{sql}"
    );
    assert!(
        !sql.contains("ON UPDATE CURRENT_TIMESTAMP"),
        "PG must not use the MySQL extra:\n{sql}"
    );
}

#[test]
fn updated_at_is_plain_on_sqlite() {
    let sql = generate_create_file(&parse_model(TS_MODEL), &DbKind::Other);
    assert!(
        !sql.contains("CREATE TRIGGER"),
        "SQLite: no trigger:\n{sql}"
    );
    assert!(
        !sql.contains("ON UPDATE CURRENT_TIMESTAMP"),
        "SQLite: no MySQL extra:\n{sql}"
    );
}

// ── Foreign key actions (relations file) ──────────────────────────────────────

#[test]
fn fk_cascade_action_is_rendered() {
    let schema = ParsedSchema {
        table_name: "comment".into(),
        primary_key: None,
        columns: vec![],
        foreign_keys: vec![ParsedFk {
            from_column: "post_id".into(),
            to_table: "post".into(),
            to_column: "id".into(),
            on_delete: "Cascade".into(),
            on_update: "NoAction".into(),
        }],
        indexes: vec![],
    };
    let sql = generate_relations_file(&[&schema]);
    assert!(
        sql.contains(".on_delete(ForeignKeyAction::Cascade)"),
        "{sql}"
    );
    assert!(
        sql.contains(r#".from(Alias::new("comment"), Alias::new("post_id"))"#),
        "{sql}"
    );
    assert!(
        sql.contains(r#".to(Alias::new("post"), Alias::new("id"))"#),
        "{sql}"
    );
}

// ── i32-backed enum: integer column, never CREATE TYPE ─────────────────────────

const ENUM_I32: &str = r#"
use runique::prelude::*;
model! {
    Task,
    table: "task",
    pk: id => Pk,
    enums: {
        Priority: i32 [Low=1, Mid=2, High=3],
    },
    {
        priority: choice [enum(Priority), required],
    }
}
"#;

#[test]
fn i32_backed_enum_is_integer_column_without_create_type() {
    let schema = parse_model(ENUM_I32);
    let prio = col(&schema, "priority");
    assert_eq!(prio.col_type, "Integer");
    assert!(
        prio.enum_string_values.is_empty(),
        "i32-backed enum carries no string variants"
    );
    let pg = generate_create_file(&schema, &DbKind::Postgres);
    assert!(
        !pg.contains("CREATE TYPE"),
        "i32 enum must not CREATE TYPE:\n{pg}"
    );
    assert!(
        pg.contains(r#".col(ColumnDef::new(Alias::new("priority")).integer()"#),
        "i32 enum is an integer column:\n{pg}"
    );
}

// ── Destructive guard matrix ──────────────────────────────────────────────────

fn empty_changes() -> Changes {
    Changes {
        table_name: "t".into(),
        added_columns: vec![],
        dropped_columns: vec![],
        modified_columns: vec![],
        renamed_columns: vec![],
        added_fks: vec![],
        dropped_fks: vec![],
        added_indexes: vec![],
        dropped_indexes: vec![],
        is_new_table: false,
        enum_renames: vec![],
        enum_value_adds: vec![],
        enum_value_drops: vec![],
    }
}

#[test]
fn destructive_type_change_is_flagged() {
    let old = ParsedColumn {
        name: "age".into(),
        col_type: "Integer".into(),
        ..Default::default()
    };
    let new = ParsedColumn {
        name: "age".into(),
        col_type: "String".into(),
        ..Default::default()
    };
    let changes = vec![Changes {
        modified_columns: vec![(old, new)],
        ..empty_changes()
    }];
    let msgs = collect_destructive_messages(&changes);
    assert!(
        msgs.iter().any(|m| m.contains("type Integer -> String")),
        "{msgs:?}"
    );
}

#[test]
fn destructive_nullable_to_not_null_is_flagged() {
    let old = ParsedColumn {
        name: "bio".into(),
        col_type: "String".into(),
        nullable: true,
        ..Default::default()
    };
    let new = ParsedColumn {
        name: "bio".into(),
        col_type: "String".into(),
        nullable: false,
        ..Default::default()
    };
    let changes = vec![Changes {
        modified_columns: vec![(old, new)],
        ..empty_changes()
    }];
    let msgs = collect_destructive_messages(&changes);
    assert!(
        msgs.iter().any(|m| m.contains("nullable -> not_null")),
        "{msgs:?}"
    );
}

#[test]
fn destructive_drop_fk_is_flagged() {
    let changes = vec![Changes {
        dropped_fks: vec![ParsedFk {
            from_column: "post_id".into(),
            to_table: "post".into(),
            to_column: "id".into(),
            on_delete: "NoAction".into(),
            on_update: "NoAction".into(),
        }],
        ..empty_changes()
    }];
    let msgs = collect_destructive_messages(&changes);
    assert!(
        msgs.iter().any(|m| m.contains("DROP FOREIGN KEY")),
        "{msgs:?}"
    );
}

#[test]
fn non_destructive_change_yields_no_messages() {
    // Adding a nullable column is safe.
    let changes = vec![Changes {
        added_columns: vec![ParsedColumn {
            name: "nickname".into(),
            col_type: "String".into(),
            nullable: true,
            ..Default::default()
        }],
        ..empty_changes()
    }];
    assert!(collect_destructive_messages(&changes).is_empty());
}

// ── Combined ALTER ordering: rename runs before drop/add ───────────────────────

#[test]
fn combined_alter_runs_rename_before_drop_and_add() {
    let changes = Changes {
        renamed_columns: vec![("old_name".into(), "new_name".into())],
        added_columns: vec![ParsedColumn {
            name: "fresh".into(),
            col_type: "Integer".into(),
            nullable: true,
            ..Default::default()
        }],
        dropped_columns: vec![ParsedColumn {
            name: "gone".into(),
            ..Default::default()
        }],
        ..empty_changes()
    };
    let sql = generate_alter_file(&changes, &DbKind::Other);
    let up = sql.split("async fn down").next().unwrap();
    let rename_pos = up.find("rename_column").expect("rename present");
    let drop_pos = up.find("drop_column").expect("drop present");
    let add_pos = up.find("add_column").expect("add present");
    assert!(
        rename_pos < drop_pos && rename_pos < add_pos,
        "RENAME must precede DROP/ADD in up:\n{sql}"
    );
}

// ── ALTER: add / drop / modify column (up + down) ──────────────────────────────

fn up_down(sql: &str) -> (&str, &str) {
    let mut parts = sql.split("async fn down");
    let up = parts.next().unwrap_or("");
    let down = parts.next().unwrap_or("");
    (up, down)
}

#[test]
fn alter_add_column_up_adds_down_drops() {
    let changes = Changes {
        added_columns: vec![ParsedColumn {
            name: "nickname".into(),
            col_type: "String".into(),
            nullable: true,
            ..Default::default()
        }],
        ..empty_changes()
    };
    let sql = generate_alter_file(&changes, &DbKind::Other);
    let (up, down) = up_down(&sql);
    assert!(
        up.contains(r#".add_column(ColumnDef::new(Alias::new("nickname")).string().null())"#),
        "up:\n{up}"
    );
    assert!(
        down.contains(r#".drop_column(Alias::new("nickname"))"#),
        "down:\n{down}"
    );
}

#[test]
fn alter_drop_column_up_drops_down_recreates() {
    let changes = Changes {
        dropped_columns: vec![ParsedColumn {
            name: "legacy".into(),
            col_type: "Integer".into(),
            ..Default::default()
        }],
        ..empty_changes()
    };
    let sql = generate_alter_file(&changes, &DbKind::Other);
    let (up, down) = up_down(&sql);
    assert!(
        up.contains(r#".drop_column(Alias::new("legacy"))"#),
        "up:\n{up}"
    );
    assert!(
        down.contains(r#".add_column(ColumnDef::new(Alias::new("legacy")).integer()"#),
        "down must recreate the dropped column:\n{down}"
    );
}

#[test]
fn alter_type_change_emits_warning_not_modify() {
    // A type change is never auto-applied (risk of data loss) — it's a manual WARNING.
    let old = ParsedColumn {
        name: "age".into(),
        col_type: "Integer".into(),
        ..Default::default()
    };
    let new = ParsedColumn {
        name: "age".into(),
        col_type: "BigInteger".into(),
        ..Default::default()
    };
    let changes = Changes {
        modified_columns: vec![(old, new)],
        ..empty_changes()
    };
    let sql = generate_alter_file(&changes, &DbKind::Other);
    assert!(
        sql.contains("WARNING: type change on column 'age': Integer -> BigInteger"),
        "type change must emit a manual WARNING:\n{sql}"
    );
    assert!(
        !sql.contains(".modify_column("),
        "type change must NOT auto-modify:\n{sql}"
    );
}

#[test]
fn alter_nullable_to_not_null_modifies_and_reverses() {
    // Same type, nullable → not_null is applied via modify_column (down restores nullable).
    let old = ParsedColumn {
        name: "bio".into(),
        col_type: "String".into(),
        nullable: true,
        ..Default::default()
    };
    let new = ParsedColumn {
        name: "bio".into(),
        col_type: "String".into(),
        nullable: false,
        ..Default::default()
    };
    let changes = Changes {
        modified_columns: vec![(old, new)],
        ..empty_changes()
    };
    let sql = generate_alter_file(&changes, &DbKind::Other);
    let (up, down) = up_down(&sql);
    assert!(
        up.contains(r#".modify_column(ColumnDef::new(Alias::new("bio")).string().not_null())"#),
        "up:\n{up}"
    );
    assert!(
        down.contains(r#".modify_column(ColumnDef::new(Alias::new("bio")).string().null())"#),
        "down must restore nullable:\n{down}"
    );
}

// ── ALTER: add / drop FK (up + down) ───────────────────────────────────────────

fn fk(from: &str, to_table: &str) -> ParsedFk {
    ParsedFk {
        from_column: from.into(),
        to_table: to_table.into(),
        to_column: "id".into(),
        on_delete: "NoAction".into(),
        on_update: "NoAction".into(),
    }
}

#[test]
fn alter_add_fk_up_creates_down_drops() {
    let changes = Changes {
        added_fks: vec![fk("post_id", "post")],
        ..empty_changes()
    };
    let sql = generate_alter_file(&changes, &DbKind::Other);
    let (up, down) = up_down(&sql);
    assert!(up.contains(".create_foreign_key("), "up:\n{up}");
    assert!(up.contains(r#".name("t_post_id_post_fkey")"#), "up:\n{up}");
    assert!(down.contains(".drop_foreign_key("), "down:\n{down}");
}

#[test]
fn alter_drop_fk_up_drops_down_creates() {
    let changes = Changes {
        dropped_fks: vec![fk("post_id", "post")],
        ..empty_changes()
    };
    let sql = generate_alter_file(&changes, &DbKind::Other);
    let (up, down) = up_down(&sql);
    assert!(up.contains(".drop_foreign_key("), "up:\n{up}");
    assert!(down.contains(".create_foreign_key("), "down:\n{down}");
}

// ── ALTER: add / drop index (up + down) ────────────────────────────────────────

#[test]
fn alter_add_unique_index_up_creates_down_drops() {
    let changes = Changes {
        added_indexes: vec![ParsedIndex {
            name: "idx_t_email".into(),
            columns: vec!["email".into()],
            unique: true,
        }],
        ..empty_changes()
    };
    let sql = generate_alter_file(&changes, &DbKind::Other);
    let (up, down) = up_down(&sql);
    assert!(up.contains(".create_index("), "up:\n{up}");
    assert!(up.contains(r#".name("idx_t_email")"#), "up:\n{up}");
    assert!(up.contains(".unique()"), "unique index:\n{up}");
    assert!(
        down.contains(r#".drop_index(Index::drop().name("idx_t_email")"#),
        "down:\n{down}"
    );
}

#[test]
fn alter_drop_index_up_drops_down_recreates() {
    let changes = Changes {
        dropped_indexes: vec![ParsedIndex {
            name: "idx_t_slug".into(),
            columns: vec!["slug".into()],
            unique: false,
        }],
        ..empty_changes()
    };
    let sql = generate_alter_file(&changes, &DbKind::Other);
    let (up, down) = up_down(&sql);
    assert!(
        up.contains(r#".drop_index(Index::drop().name("idx_t_slug")"#),
        "up:\n{up}"
    );
    assert!(down.contains(".create_index("), "down:\n{down}");
    assert!(
        !down.contains(".unique()"),
        "non-unique index must not be unique on recreate:\n{down}"
    );
}

// ── Column rendering: defaults, unique, type matrix ────────────────────────────

const STR_DEFAULT: &str = r#"
use runique::prelude::*;
model! {
    Cfg,
    table: "cfg",
    pk: id => Pk,
    {
        role: text [default: "guest"],
    }
}
"#;

#[test]
fn string_literal_default_is_emitted() {
    let sql = generate_create_file(&parse_model(STR_DEFAULT), &DbKind::Other);
    assert!(sql.contains(r#".default("guest")"#), "{sql}");
}

const UNIQUE_COL: &str = r#"
use runique::prelude::*;
model! {
    Acc,
    table: "acc",
    pk: id => Pk,
    {
        email: text [required, unique],
    }
}
"#;

#[test]
fn unique_required_column_renders_unique_key_not_null() {
    let sql = generate_create_file(&parse_model(UNIQUE_COL), &DbKind::Other);
    assert!(
        sql.contains(
            r#".col(ColumnDef::new(Alias::new("email")).string().not_null().unique_key())"#
        ),
        "{sql}"
    );
}

const TYPES_MODEL: &str = r#"
use runique::prelude::*;
model! {
    Mix,
    table: "mix",
    pk: id => Pk,
    {
        price:   decimal,
        active:  bool,
        ref_id:  uuid,
        payload: json,
        big:     bigint,
        ratio:   float,
    }
}
"#;

#[test]
fn semantic_types_map_to_seaorm_methods() {
    let sql = generate_create_file(&parse_model(TYPES_MODEL), &DbKind::Other);
    for needle in [
        ".decimal()",
        ".boolean()",
        ".uuid()",
        ".json()",
        ".big_integer()",
        ".double()",
    ] {
        assert!(sql.contains(needle), "missing `{needle}`:\n{sql}");
    }
}

// ── Changes::is_empty ──────────────────────────────────────────────────────────

#[test]
fn empty_changes_is_empty() {
    assert!(empty_changes().is_empty());
}

#[test]
fn new_table_is_not_empty() {
    let c = Changes {
        is_new_table: true,
        ..empty_changes()
    };
    assert!(!c.is_empty(), "a new table is a change");
}

#[test]
fn a_lone_rename_is_not_empty() {
    let c = Changes {
        renamed_columns: vec![("a".into(), "b".into())],
        ..empty_changes()
    };
    assert!(!c.is_empty(), "a rename is a change");
}

// ── Multiple renames in one ALTER ──────────────────────────────────────────────

#[test]
fn multiple_column_renames_all_emitted() {
    let changes = Changes {
        renamed_columns: vec![
            ("first".into(), "given_name".into()),
            ("last".into(), "family_name".into()),
        ],
        ..empty_changes()
    };
    let sql = generate_alter_file(&changes, &DbKind::Other);
    assert!(
        sql.contains(r#".rename_column(Alias::new("first"), Alias::new("given_name"))"#),
        "{sql}"
    );
    assert!(
        sql.contains(r#".rename_column(Alias::new("last"), Alias::new("family_name"))"#),
        "{sql}"
    );
}

// ── FK actions: SetNull / Restrict (relations file) ────────────────────────────

#[test]
fn fk_set_null_and_restrict_actions_render() {
    let schema = ParsedSchema {
        table_name: "child".into(),
        primary_key: None,
        columns: vec![],
        foreign_keys: vec![ParsedFk {
            from_column: "a_id".into(),
            to_table: "a".into(),
            to_column: "id".into(),
            on_delete: "SetNull".into(),
            on_update: "Restrict".into(),
        }],
        indexes: vec![],
    };
    let sql = generate_relations_file(&[&schema]);
    assert!(
        sql.contains(".on_delete(ForeignKeyAction::SetNull)"),
        "{sql}"
    );
    assert!(
        sql.contains(".on_update(ForeignKeyAction::Restrict)"),
        "{sql}"
    );
}

// ── FK snapshot round-trip is stable ───────────────────────────────────────────

#[test]
fn fk_round_trip_is_stable() {
    let schema = ParsedSchema {
        table_name: "comment".into(),
        primary_key: Some(ParsedColumn {
            name: "id".into(),
            col_type: "Integer".into(),
            ..Default::default()
        }),
        columns: vec![ParsedColumn {
            name: "post_id".into(),
            col_type: "Integer".into(),
            ..Default::default()
        }],
        foreign_keys: vec![ParsedFk {
            from_column: "post_id".into(),
            to_table: "post".into(),
            to_column: "id".into(),
            on_delete: "Cascade".into(),
            on_update: "NoAction".into(),
        }],
        indexes: vec![],
    };
    let snapshot = generate_snapshot_file(&schema);
    let reparsed = parse_seaorm_source(&snapshot).expect("snapshot should reparse");
    let changes = diff_schemas(&reparsed, &schema);
    assert!(
        changes.added_fks.is_empty() && changes.dropped_fks.is_empty(),
        "FK round-trip not stable: {changes:?}"
    );
}

#[test]
fn sqlite_inlines_fk_in_create_table_but_pg_does_not() {
    // SQLite cannot ALTER-ADD FKs, so they are inlined in CREATE TABLE.
    // PG/MySQL keep them in the separate relations migration.
    let schema = ParsedSchema {
        table_name: "comment".into(),
        primary_key: Some(ParsedColumn {
            name: "id".into(),
            col_type: "Integer".into(),
            ..Default::default()
        }),
        columns: vec![ParsedColumn {
            name: "post_id".into(),
            col_type: "Integer".into(),
            ..Default::default()
        }],
        foreign_keys: vec![ParsedFk {
            from_column: "post_id".into(),
            to_table: "post".into(),
            to_column: "id".into(),
            on_delete: "Cascade".into(),
            on_update: "NoAction".into(),
        }],
        indexes: vec![],
    };

    let sqlite = generate_create_file(&schema, &DbKind::Other);
    assert!(
        sqlite.contains(".foreign_key("),
        "SQLite must inline FK in CREATE:\n{sqlite}"
    );
    assert!(
        sqlite.contains(r#".from(Alias::new("comment"), Alias::new("post_id"))"#),
        "inline FK from clause:\n{sqlite}"
    );
    assert!(
        sqlite.contains(".on_delete(ForeignKeyAction::Cascade)"),
        "inline FK action:\n{sqlite}"
    );

    let pg = generate_create_file(&schema, &DbKind::Postgres);
    assert!(
        !pg.contains(".foreign_key("),
        "PG must NOT inline FK in CREATE:\n{pg}"
    );

    // The snapshot never inlines (keeps round-trip stable).
    let snap = generate_snapshot_file(&schema);
    assert!(
        !snap.contains(".foreign_key("),
        "snapshot must not inline FK:\n{snap}"
    );
}

// ── No-op diff ─────────────────────────────────────────────────────────────────

#[test]
fn identical_schemas_produce_no_changes() {
    let changes = diff_schemas(&parse_model(TYPES_MODEL), &parse_model(TYPES_MODEL));
    assert!(
        changes.is_empty(),
        "identical schemas must diff to empty: {changes:?}"
    );
}

// ── Ignored columns (readonly) are excluded from migrations ────────────────────

const IGNORED_COL: &str = r#"
use runique::prelude::*;
model! {
    Sec,
    table: "sec",
    pk: id => Pk,
    {
        visible: text,
        note: text [readonly],
    }
}
"#;

#[test]
fn readonly_column_is_excluded_from_migration() {
    let schema = parse_model(IGNORED_COL);
    assert!(
        col(&schema, "note").ignored,
        "readonly column must be flagged ignored"
    );
    let sql = generate_create_file(&schema, &DbKind::Other);
    assert!(
        sql.contains(r#"Alias::new("visible")"#),
        "visible column expected:\n{sql}"
    );
    assert!(
        !sql.contains(r#"Alias::new("note")"#),
        "ignored column must not appear in the migration:\n{sql}"
    );
}

// ── Date / time / timestamptz column methods ───────────────────────────────────

const DT_TYPES: &str = r#"
use runique::prelude::*;
model! {
    Sched,
    table: "sched",
    pk: id => Pk,
    {
        d:  date,
        t:  time,
        tz: timestamp_tz,
    }
}
"#;

#[test]
fn date_time_types_map_to_methods() {
    let sql = generate_create_file(&parse_model(DT_TYPES), &DbKind::Other);
    assert!(sql.contains(r#"Alias::new("d")).date()"#), "date:\n{sql}");
    assert!(sql.contains(r#"Alias::new("t")).time()"#), "time:\n{sql}");
    assert!(
        sql.contains(r#"Alias::new("tz")).timestamp_tz()"#),
        "timestamptz:\n{sql}"
    );
}

// ── ALTER: add UNIQUE via safe modify ──────────────────────────────────────────

#[test]
fn alter_add_unique_via_modify_column() {
    // Same type, same nullability, unique false → true → applied via modify_column.
    let old = ParsedColumn {
        name: "email".into(),
        col_type: "String".into(),
        nullable: false,
        unique: false,
        ..Default::default()
    };
    let new = ParsedColumn {
        name: "email".into(),
        col_type: "String".into(),
        nullable: false,
        unique: true,
        ..Default::default()
    };
    let changes = Changes {
        modified_columns: vec![(old, new)],
        ..empty_changes()
    };
    let sql = generate_alter_file(&changes, &DbKind::Other);
    let (up, down) = up_down(&sql);
    assert!(
        up.contains(r#".modify_column(ColumnDef::new(Alias::new("email")).string().not_null().unique_key())"#),
        "up:\n{up}"
    );
    assert!(
        down.contains(r#".modify_column(ColumnDef::new(Alias::new("email")).string().not_null())"#),
        "down must drop the unique:\n{down}"
    );
}

// ── ALTER: multiple added columns ──────────────────────────────────────────────

#[test]
fn alter_multiple_added_columns_all_present() {
    let changes = Changes {
        added_columns: vec![
            ParsedColumn {
                name: "a".into(),
                col_type: "Integer".into(),
                nullable: true,
                ..Default::default()
            },
            ParsedColumn {
                name: "b".into(),
                col_type: "String".into(),
                nullable: true,
                ..Default::default()
            },
        ],
        ..empty_changes()
    };
    let sql = generate_alter_file(&changes, &DbKind::Other);
    assert!(
        sql.contains(r#".add_column(ColumnDef::new(Alias::new("a")).integer().null())"#),
        "{sql}"
    );
    assert!(
        sql.contains(r#".add_column(ColumnDef::new(Alias::new("b")).string().null())"#),
        "{sql}"
    );
}

// ── Enum value DROP: WARNING (up) + re-add (down), Postgres only ────────────────

#[test]
fn enum_value_drop_warns_up_and_readds_down_on_postgres() {
    let changes = Changes {
        enum_value_drops: vec![("status".into(), "Status".into(), "Legacy".into())],
        ..empty_changes()
    };
    let pg = generate_alter_file(&changes, &DbKind::Postgres);
    let (up, down) = up_down(&pg);
    assert!(
        up.contains("WARNING: value 'Legacy' removed"),
        "up warning:\n{up}"
    );
    assert!(
        down.contains("ALTER TYPE Status ADD VALUE IF NOT EXISTS 'Legacy'"),
        "down must re-add the value:\n{down}"
    );

    // Non-PG: VARCHAR enums need no DDL for value drops.
    let other = generate_alter_file(&changes, &DbKind::Other);
    assert!(
        !other.contains("WARNING"),
        "non-PG must not emit enum-drop handling:\n{other}"
    );
}

// ── Enum column default: emitted on the Enum coldef, all engines ───────────────
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
    for kind in [DbKind::Postgres, DbKind::Mysql, DbKind::Other] {
        let sql = generate_create_file(&schema, &kind);
        assert!(
            sql.contains("ColumnType::Enum"),
            "enum coldef missing for {kind:?}:\n{sql}"
        );
        assert!(
            sql.contains(r#".not_null().default("Draft")"#),
            "enum default dropped for {kind:?}:\n{sql}"
        );
    }
}

const EXTEND_ENUM_DEFAULT_SRC: &str = r#"
use runique::prelude::*;
extend! {
    table: "blog",
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
    let schema = &parse_extend_blocks_from_source(EXTEND_ENUM_DEFAULT_SRC)[0];
    let change = Changes {
        table_name: schema.table_name.clone(),
        added_columns: schema.columns.clone(),
        dropped_columns: vec![],
        modified_columns: vec![],
        added_fks: vec![],
        dropped_fks: vec![],
        added_indexes: vec![],
        dropped_indexes: vec![],
        is_new_table: false,
        renamed_columns: vec![],
        enum_renames: vec![],
        enum_value_adds: vec![],
        enum_value_drops: vec![],
    };
    for kind in [DbKind::Postgres, DbKind::Mysql, DbKind::Other] {
        let sql = generate_alter_file(&change, &kind);
        assert!(
            sql.contains(r#".not_null().default("Draft")"#),
            "extend enum default missing on ADD COLUMN for {kind:?}:\n{sql}"
        );
    }
    // CREATE TYPE stays Postgres-only.
    assert!(
        generate_alter_file(&change, &DbKind::Postgres).contains("CREATE TYPE"),
        "PG must still CREATE TYPE for the enum"
    );
    assert!(
        !generate_alter_file(&change, &DbKind::Other).contains("CREATE TYPE"),
        "non-PG must not CREATE TYPE"
    );
}

// ── Topological sort: referenced tables created first ──────────────────────────

fn new_table(name: &str, fks: Vec<ParsedFk>) -> Changes {
    Changes {
        table_name: name.into(),
        is_new_table: true,
        added_fks: fks,
        ..empty_changes()
    }
}

#[test]
fn topological_sort_creates_referenced_table_first() {
    // `comment` has a FK to `post`; both are new → `post` must be created first.
    let sorted = topological_sort_changes(vec![
        new_table("comment", vec![fk("post_id", "post")]),
        new_table("post", vec![]),
    ]);
    let order: Vec<&str> = sorted.iter().map(|c| c.table_name.as_str()).collect();
    let post = order.iter().position(|t| *t == "post").unwrap();
    let comment = order.iter().position(|t| *t == "comment").unwrap();
    assert!(
        post < comment,
        "referenced table must come first: {order:?}"
    );
}

#[test]
fn topological_sort_orders_a_fk_chain() {
    // a → b → c : creation order must be c, then b, then a.
    let sorted = topological_sort_changes(vec![
        new_table("a", vec![fk("b_id", "b")]),
        new_table("b", vec![fk("c_id", "c")]),
        new_table("c", vec![]),
    ]);
    let order: Vec<&str> = sorted.iter().map(|c| c.table_name.as_str()).collect();
    let pos = |t: &str| order.iter().position(|x| *x == t).unwrap();
    assert!(
        pos("c") < pos("b") && pos("b") < pos("a"),
        "chain order wrong: {order:?}"
    );
}

#[test]
fn topological_sort_ignores_fk_to_existing_table() {
    // A FK to a table NOT created in this batch does not constrain ordering (no panic, kept).
    let sorted = topological_sort_changes(vec![new_table("comment", vec![fk("user_id", "users")])]);
    assert_eq!(sorted.len(), 1);
    assert_eq!(sorted[0].table_name, "comment");
}
