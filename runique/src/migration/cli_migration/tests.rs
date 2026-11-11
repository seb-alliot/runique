//! Written from cargo-mutants survivors (2026-10-08): the private steps of
//! `run` — snapshot upgrades, lib.rs editing, ordering, guards.
use super::lib_file::*;
use super::plan::*;
use super::report::*;
use super::snapshot::*;
use super::*;
use crate::migration::utils::types::{ParsedColumn, ParsedFk};
use std::fs;
use std::path::Path;

/// Only a plain type change is listed: a change to or from an enum is
/// migrated, and a column whose other attributes changed isn't concerned.
#[test]
fn only_plain_type_changes_are_reported_as_unmigrated() {
    let col = |col_type: &str, enum_values: &[&str], nullable: bool| ParsedColumn {
        name: "c".into(),
        col_type: col_type.into(),
        nullable,
        enum_string_values: enum_values.iter().map(|v| v.to_string()).collect(),
        ..ParsedColumn::default()
    };
    let changes = |old: ParsedColumn, new: ParsedColumn| Changes {
        table_name: "t".into(),
        added_columns: vec![],
        dropped_columns: vec![],
        modified_columns: vec![(old, new)],
        renamed_columns: vec![],
        added_fks: vec![],
        dropped_fks: vec![],
        added_indexes: vec![],
        dropped_indexes: vec![],
        is_new_table: false,
        enum_renames: vec![],
        enum_value_adds: vec![],
        enum_value_drops: vec![],
    };

    let plain = changes(col("Integer", &[], false), col("BigInteger", &[], false));
    assert_eq!(
        unmigrated_type_changes(&[plain]),
        vec!["  t.c: Integer -> BigInteger".to_string()]
    );

    let to_enum = changes(col("String", &[], false), col("Enum", &["a"], false));
    assert!(unmigrated_type_changes(&[to_enum]).is_empty());

    let nullable = changes(col("String", &[], true), col("String", &[], false));
    assert!(unmigrated_type_changes(&[nullable]).is_empty());
}

fn temp(tag: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!("rq_mk_{tag}_{}", uuid::Uuid::new_v4()));
    fs::create_dir_all(&dir).unwrap();
    dir
}

fn schema(table: &str, fks: &[&str]) -> ParsedSchema {
    ParsedSchema {
        table_name: table.to_string(),
        primary_key: Some(ParsedColumn {
            name: "id".into(),
            col_type: "Integer".into(),
            ..ParsedColumn::default()
        }),
        columns: vec![ParsedColumn {
            name: "title".into(),
            col_type: "String".into(),
            max_length: Some(50),
            ..ParsedColumn::default()
        }],
        foreign_keys: fks
            .iter()
            .map(|to| ParsedFk {
                from_column: format!("{to}_id"),
                to_table: (*to).to_string(),
                to_column: "id".into(),
                on_delete: "Cascade".into(),
                on_update: "NoAction".into(),
            })
            .collect(),
        indexes: vec![],
    }
}

fn new_table(s: &ParsedSchema) -> Changes {
    Changes {
        table_name: s.table_name.clone(),
        added_columns: s.columns.clone(),
        dropped_columns: vec![],
        modified_columns: vec![],
        renamed_columns: vec![],
        added_fks: s.foreign_keys.clone(),
        dropped_fks: vec![],
        added_indexes: vec![],
        dropped_indexes: vec![],
        is_new_table: true,
        enum_renames: vec![],
        enum_value_adds: vec![],
        enum_value_drops: vec![],
    }
}

#[test]
fn an_extend_snapshot_is_marked_as_recording_lengths() {
    let content = extend_snapshot_content(&schema("eihwaz_users", &[]));
    assert!(content.starts_with(SNAPSHOT_LENGTHS_MARKER), "{content}");
    assert!(content.contains("create_table"), "{content}");
}

/// Only an old-format snapshot of a table this run doesn't touch is rewritten.
#[test]
fn only_untouched_old_snapshots_are_upgraded() {
    let (entities, migrations) = (temp("ent"), temp("mig"));
    let (e, m) = (entities.to_str().unwrap(), migrations.to_str().unwrap());
    fs::create_dir_all(snapshot_dir(m)).unwrap();
    let tables = [
        schema("old_untouched", &[]),
        schema("old_changed", &[]),
        schema("recorded", &[]),
    ];
    fs::write(snapshot_file_path(m, "old_untouched"), "// old").unwrap();
    fs::write(snapshot_file_path(m, "old_changed"), "// old").unwrap();
    fs::write(
        snapshot_file_path(m, "recorded"),
        generate_snapshot_file(&tables[2]),
    )
    .unwrap();
    // `extend!{}` side: an old snapshot of an extension nobody re-planned.
    fs::write(
        entities.join("ext.rs"),
        r#"extend! { table: "eihwaz_users", fields: { bio: text } }"#,
    )
    .unwrap();
    fs::create_dir_all(extend_snapshot_dir(m)).unwrap();
    fs::write(extend_snapshot_file_path(m, "eihwaz_users"), "// old").unwrap();

    let changed = [new_table(&tables[1])];
    let upgrades = snapshot_upgrades(e, &tables, &changed, &[], m).unwrap();
    let paths: Vec<&str> = upgrades.iter().map(|(p, _)| p.as_str()).collect();
    assert_eq!(
        paths,
        [
            snapshot_file_path(m, "old_untouched").as_str(),
            extend_snapshot_file_path(m, "eihwaz_users").as_str()
        ]
    );
    assert!(upgrades[1].1.starts_with(SNAPSHOT_LENGTHS_MARKER));

    // Re-planned extension: left to its own migration.
    let ext = merge_extend_schemas(scan_extend_blocks(e).unwrap()).remove(0);
    let planned = [(ext.clone(), new_table(&ext))];
    let upgrades = snapshot_upgrades(e, &tables, &changed, &planned, m).unwrap();
    assert_eq!(upgrades.len(), 1);
    let _ = (
        fs::remove_dir_all(&entities),
        fs::remove_dir_all(&migrations),
    );
}

/// The `vec![...]` of lib.rs is found by bracket depth and read entry by
/// entry, whatever the layout.
#[test]
fn lib_entries_are_read_whatever_the_layout() {
    let content = "fn m() -> Vec<X> { vec![Box::new(a::Migration), Box::new(b::Migration)] }";
    let (s, e) = find_vec_span(content).unwrap();
    assert_eq!(
        &content[s..e],
        "[Box::new(a::Migration), Box::new(b::Migration)]"
    );
    assert_eq!(
        extract_box_entries(&content[s..e]),
        ["Box::new(a::Migration)", "Box::new(b::Migration)"]
    );
    assert_eq!(find_vec_span("no list here"), None);
}

/// No model of the project's own yet: the framework's migrations still
/// land in the `lib.rs` that `sea-orm-cli migrate init` wrote, its
/// placeholder gone — otherwise `migrate up` doesn't even compile.
#[test]
fn without_any_model_the_framework_migrations_are_wired_in() {
    let (entities, migrations) = (temp("ent_none"), temp("mig_none"));
    let (e, m) = (entities.to_str().unwrap(), migrations.to_str().unwrap());
    fs::write(
        migrations.join("lib.rs"),
        "pub use sea_orm_migration::prelude::*;\n\nmod m20220101_000001_create_table;\n\n\
         pub struct Migrator;\n\n#[async_trait::async_trait]\nimpl MigratorTrait for Migrator {\n    \
         fn migrations() -> Vec<Box<dyn MigrationTrait>> {\n        \
         vec![Box::new(m20220101_000001_create_table::Migration)]\n    }\n}\n",
    )
    .unwrap();
    fs::write(
        migrations.join("m20220101_000001_create_table.rs"),
        "// placeholder",
    )
    .unwrap();

    run(e, m, false).unwrap();
    let lib = fs::read_to_string(migrations.join("lib.rs")).unwrap();
    let placeholder_file = migrations.join("m20220101_000001_create_table.rs").exists();
    let _ = (
        fs::remove_dir_all(&entities),
        fs::remove_dir_all(&migrations),
    );

    assert!(
        lib.contains("migrations_table::EihwazUsersMigration"),
        "{lib}"
    );
    // `AdminTableMigration` no longer creates the history: its own
    // migration must be there, or a new database has no history table.
    assert!(
        lib.contains("migrations_table::EihwazHistoryMigration"),
        "{lib}"
    );
    assert!(!lib.contains("m20220101_000001_create_table"), "{lib}");
    assert!(!placeholder_file);
}

#[test]
fn the_sea_orm_cli_placeholder_is_removed() {
    let dir = temp("placeholder");
    let m = dir.to_str().unwrap();
    let file = format!("{m}/{SEA_ORM_CLI_PLACEHOLDER_MODULE}.rs");
    fs::write(&file, "// placeholder").unwrap();
    fs::write(dir.join("keep.rs"), "// mine").unwrap();
    strip_sea_orm_cli_placeholder_file(m);
    let (gone, kept) = (!Path::new(&file).exists(), dir.join("keep.rs").exists());
    let _ = fs::remove_dir_all(&dir);
    assert!(gone && kept);
}

/// A key to an existing table never holds a new table back.
#[test]
fn existing_targets_do_not_change_the_creation_order() {
    let zeta = schema("zeta", &["users"]);
    let alpha = schema("alpha", &["zeta"]);
    let sorted = topological_sort_changes(vec![new_table(&alpha), new_table(&zeta)]);
    let names: Vec<&str> = sorted.iter().map(|c| c.table_name.as_str()).collect();
    assert_eq!(names, ["zeta", "alpha"]);
}

#[test]
fn length_shrinks_at_their_boundaries() {
    assert!(length_may_shrink(Some(100), Some(50)));
    assert!(!length_may_shrink(Some(50), Some(50)), "same length");
    assert!(!length_may_shrink(Some(50), Some(100)));
    assert!(length_may_shrink(Some(300), None), "MySQL's VARCHAR(255)");
    assert!(!length_may_shrink(Some(255), None));
    assert!(!length_may_shrink(Some(100), None));
}

#[test]
fn decimal_shrinks_at_their_boundaries() {
    assert!(
        !decimal_may_shrink(Some((10, 2)), Some((10, 2))),
        "same precision"
    );
    assert!(
        !decimal_may_shrink(Some((10, 2)), Some((12, 4))),
        "both sides grow"
    );
    assert!(
        decimal_may_shrink(Some((10, 2)), Some((10, 1))),
        "fewer places"
    );
    assert!(
        decimal_may_shrink(Some((10, 2)), Some((10, 3))),
        "fewer integer digits"
    );
    assert!(decimal_may_shrink(None, Some((10, 2))), "first precision");
    assert!(!decimal_may_shrink(Some((10, 2)), None));
}

/// 63 characters is the Postgres limit: 63 passes, 64 is refused.
#[test]
fn identifier_lengths_at_the_postgres_limit() {
    let mut change = new_table(&schema("t", &[]));
    let fk_to = |len: usize| ParsedFk {
        // "{table}_{from}_{to}_fkey" with table "t", from "c".
        to_table: "x".repeat(len - "t_c__fkey".len()),
        from_column: "c".into(),
        to_column: "id".into(),
        on_delete: "Cascade".into(),
        on_update: "NoAction".into(),
    };
    let index = |len: usize| crate::migration::utils::types::ParsedIndex {
        name: "i".repeat(len),
        columns: vec!["title".into()],
        unique: false,
    };
    change.added_fks = vec![fk_to(63)];
    change.added_indexes = vec![index(63)];
    assert!(collect_long_identifier_messages(std::slice::from_ref(&change)).is_empty());
    assert!(check_identifier_lengths(std::slice::from_ref(&change)).is_ok());
    change.added_fks = vec![fk_to(64)];
    change.added_indexes = vec![index(64)];
    assert_eq!(
        collect_long_identifier_messages(std::slice::from_ref(&change)).len(),
        2
    );
    assert!(check_identifier_lengths(&[change]).is_err());
}

/// In a cycle of new tables, the key to a table created later waits for
/// it; a self key or a key to an earlier table doesn't.
#[test]
fn only_keys_to_later_new_tables_wait() {
    let schemas = [schema("a", &["b", "a"]), schema("b", &["a"])];
    let changes = [new_table(&schemas[0]), new_table(&schemas[1])];
    let keys = cycle_keys(&changes, &schemas);
    let forward: Vec<&str> = keys["a"]
        .forward
        .iter()
        .map(|f| f.to_table.as_str())
        .collect();
    assert_eq!(forward, ["b"], "self key and earlier table excluded");
    assert!(
        keys.get("b").is_none_or(|k| k.forward.is_empty()),
        "a is created before b"
    );
    assert_eq!(keys["b"].closing.len(), 1);
}

#[test]
fn extend_blocks_are_planned_until_their_snapshot_matches() {
    let (entities, migrations) = (temp("ext_ent"), temp("ext_mig"));
    let (e, m) = (entities.to_str().unwrap(), migrations.to_str().unwrap());
    fs::write(
        entities.join("ext.rs"),
        r#"extend! { table: "eihwaz_users", fields: { bio: text } }"#,
    )
    .unwrap();
    let planned = plan_extend_changes(e, m).unwrap();
    assert_eq!(planned.len(), 1);
    assert_eq!(planned[0].1.added_columns[0].name, "bio");
    let _ = (
        fs::remove_dir_all(&entities),
        fs::remove_dir_all(&migrations),
    );
}
