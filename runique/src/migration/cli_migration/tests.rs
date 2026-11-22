//! Written from cargo-mutants survivors (2026-10-08): the private steps of
//! `run` — snapshot upgrades, lib.rs editing, ordering, guards.
use super::lib_file::*;
use super::plan::*;
use super::report::*;
use super::snapshot::*;
use super::*;
use crate::migration::types::{ParsedColumn, ParsedFk};
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

// A marker is a whole line, wherever it is; a line that only quotes it is not.
#[test]
fn a_marker_is_a_line_of_its_own_wherever_it_is() {
    let m = WAIT_MIGRATION;
    for content in [
        format!("{m}\nfn up() {{}}\n"),
        format!("fn up() {{}}\n{m}\n"),
        format!("fn up() {{}}\n{m}"),
        format!("fn up() {{}}\n \t{m}  \r\nfn down() {{}}\n"),
    ] {
        assert!(has_marker(&content, m), "{content:?}");
    }
    for content in [
        String::new(),
        "fn up() {}\n".to_string(),
        format!("{m}, as the doc says\n"),
        format!("// see {m}\n"),
        format!("{m}{m}\n"),
    ] {
        assert!(!has_marker(&content, m), "{content:?}");
        assert_eq!(without_marker(&content, m), content, "{content:?}");
    }
}

// Every line that is the marker goes, and only those: the other lines keep
// their own line endings.
#[test]
fn removing_a_marker_leaves_the_rest_byte_for_byte() {
    let m = WAIT_MIGRATION;
    let content = format!("a\r\n{m}\r\n  b\n// see {m}\n{m}\nc");
    assert_eq!(
        without_marker(&content, m),
        format!("a\r\n  b\n// see {m}\nc")
    );
    assert_eq!(without_marker(&content, SNAPSHOT_LENGTHS_MARKER), content);
}

// The lengths marker moved below a header still says the snapshot records
// lengths: its own are kept. Without the marker they are the model's.
#[test]
fn a_moved_lengths_marker_still_marks_the_snapshot() {
    let dir = TempDir::new();
    let recorded = generate_snapshot_file(&schema("menus", &[]));
    let mut longer = schema("menus", &[]);
    longer.columns[0].max_length = Some(80);
    let length_read_from = |content: String| {
        let path = dir.join("menus.rs");
        fs::write(&path, content).unwrap();
        previous_snapshot(path.to_str().unwrap(), &longer)
            .unwrap()
            .columns[0]
            .max_length
    };

    assert_eq!(length_read_from(recorded.clone()), Some(50));
    assert_eq!(
        length_read_from(format!("// header\n\n{recorded}")),
        Some(50)
    );
    assert_eq!(
        length_read_from(without_marker(&recorded, SNAPSHOT_LENGTHS_MARKER)),
        Some(80)
    );
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
    let index = |len: usize| crate::migration::types::ParsedIndex {
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

#[test]
fn an_extend_change_enters_the_plan_with_its_module_and_no_snapshot() {
    let (entities, migrations) = (temp("ext_plan_ent"), temp("ext_plan_mig"));
    let (e, m) = (entities.to_str().unwrap(), migrations.to_str().unwrap());
    fs::write(
        entities.join("ext.rs"),
        r#"extend! { table: "eihwaz_users", fields: { bio: text } }"#,
    )
    .unwrap();
    let planned = plan_extend_changes(e, m).unwrap();
    let _ = (
        fs::remove_dir_all(&entities),
        fs::remove_dir_all(&migrations),
    );

    let mut plan = Plan::default();
    build_extend_plan(&mut plan, &planned, m, "20260101_000000", &[]);

    let paths: Vec<&str> = plan.files.iter().map(|(path, _)| path.as_str()).collect();
    assert_eq!(
        paths,
        [seaorm_extend_file_path(
            m,
            "20260101_000000",
            "eihwaz_users"
        )]
    );
    assert!(plan.files[0].1.contains("bio"), "{}", plan.files[0].1);
    assert_eq!(
        plan.lib_modules,
        ["m20260101_000000_extend_eihwaz_users_table"]
    );

    // Nothing to extend: the plan is left as it was.
    let mut empty = Plan::default();
    build_extend_plan(&mut empty, &[], m, "20260101_000000", &[]);
    assert!(empty.files.is_empty() && empty.lib_modules.is_empty());
}

// ── The tag of a migration nothing applied yet ───────────────────────────────

// Created, altered or extended: the migration starts with the tag, on a line
// of its own. The plan holds no snapshot: `migration up` writes it.
#[test]
fn every_generated_migration_starts_with_the_waiting_tag() {
    let (menus, plats, users) = (
        schema("menus", &[]),
        schema("plats", &[]),
        schema("eihwaz_users", &[]),
    );
    let altered = |s: &ParsedSchema| Changes {
        is_new_table: false,
        ..new_table(s)
    };
    let tag = format!("{WAIT_MIGRATION}\n");

    let mut plan = Plan::default();
    build_main_plan(
        &mut plan,
        &[new_table(&menus), altered(&plats)],
        &[menus.clone(), plats.clone()],
        "m",
        "20260101_000000",
        &[],
    );
    build_extend_plan(
        &mut plan,
        &[(users.clone(), altered(&users))],
        "m",
        "20260101_000000",
        &[],
    );

    assert_eq!(
        plan.lib_modules,
        [
            "m20260101_000000_create_menus_table",
            "m20260101_000000_alter_plats_table",
            "m20260101_000000_extend_eihwaz_users_table",
        ]
    );
    assert_eq!(plan.files.len(), 3);
    for (path, migration) in &plan.files {
        assert!(!path.contains("/snapshots/"), "{path}");
        assert!(migration.starts_with(&tag), "{path}: {migration}");
        assert_eq!(migration.matches(WAIT_MIGRATION).count(), 1, "{path}");
    }
}

// The tag is a comment: a tagged migration is read back as before.
#[test]
fn a_tagged_migration_is_still_read_back() {
    let menus = schema("menus", &[]);
    let mut plan = Plan::default();
    build_main_plan(
        &mut plan,
        &[new_table(&menus)],
        std::slice::from_ref(&menus),
        "m",
        "20260101_000000",
        &[],
    );

    let read = parse_seaorm_source(&plan.files[0].1).unwrap();
    assert_eq!(read.table_name, "menus");
    assert!(read.columns.iter().any(|c| c.name == "title"));
}

// ── `commit_plan`: the plan written as one batch ─────────────────────────────

use crate::atomic::tests::TempDir;

/// A `migration/src` folder holding `lib`, when given; its path.
fn migrations_src(dir: &TempDir, lib: Option<&str>) -> String {
    let src = dir.join("src");
    fs::create_dir_all(&src).unwrap();
    let path = src.to_str().unwrap().to_string();
    if let Some(content) = lib {
        fs::write(lib_path(&path), content).unwrap();
    }
    path
}

fn file(path: &str, name: &str, content: &str) -> (String, String) {
    (format!("{path}/{name}"), content.to_string())
}

#[test]
fn a_plan_writes_its_files_and_registers_its_modules() {
    let dir = TempDir::new();
    let path = migrations_src(&dir, Some(&initial_lib()));
    let plan = Plan {
        files: vec![
            file(&path, "m1_create_shop_table.rs", "// shop"),
            file(&path, "snapshots/shop.rs", "// snapshot"),
        ],
        lib_modules: vec!["m1_create_shop_table".to_string()],
        ..Plan::default()
    };

    commit_plan(&plan, &path).unwrap();

    let lib = fs::read_to_string(lib_path(&path)).unwrap();
    assert!(lib.contains("mod m1_create_shop_table;"), "{lib}");
    assert!(
        lib.contains("Box::new(m1_create_shop_table::Migration)"),
        "{lib}"
    );
    assert!(
        lib.contains("migrations_table::EihwazUsersMigration"),
        "{lib}"
    );
    assert_eq!(
        fs::read_to_string(format!("{path}/snapshots/shop.rs")).unwrap(),
        "// snapshot"
    );
    assert_eq!(
        dir.entries(),
        [
            "src",
            "src/lib.rs",
            "src/m1_create_shop_table.rs",
            "src/snapshots",
            "src/snapshots/shop.rs"
        ]
    );
}

// A directory sits where the last file of the plan goes: by then the snapshot
// has been replaced and the migration file placed. Everything comes back.
#[cfg(unix)]
#[test]
fn a_plan_that_fails_leaves_the_project_as_it_was() {
    let dir = TempDir::new();
    let before = initial_lib();
    let path = migrations_src(&dir, Some(&before));
    fs::create_dir(format!("{path}/snapshots")).unwrap();
    fs::write(format!("{path}/snapshots/shop.rs"), "// old snapshot").unwrap();
    fs::create_dir(format!("{path}/blocked.rs")).unwrap();
    let plan = Plan {
        files: vec![
            file(&path, "snapshots/shop.rs", "// new snapshot"),
            file(&path, "m1_create_shop_table.rs", "// shop"),
            file(&path, "blocked.rs", "// never written"),
        ],
        lib_modules: vec!["m1_create_shop_table".to_string()],
        ..Plan::default()
    };

    let error = commit_plan(&plan, &path).unwrap_err();

    assert!(format!("{error:#}").contains("blocked.rs"), "{error:#}");
    assert_eq!(fs::read_to_string(lib_path(&path)).unwrap(), before);
    assert_eq!(
        fs::read_to_string(format!("{path}/snapshots/shop.rs")).unwrap(),
        "// old snapshot"
    );
    assert_eq!(
        dir.entries(),
        [
            "src",
            "src/blocked.rs",
            "src/lib.rs",
            "src/snapshots",
            "src/snapshots/shop.rs"
        ]
    );
}

#[test]
fn a_plan_naming_one_file_twice_is_refused_and_writes_nothing() {
    let dir = TempDir::new();
    let before = initial_lib();
    let path = migrations_src(&dir, Some(&before));
    let plan = Plan {
        files: vec![
            file(&path, "m1_create_shop_table.rs", "// first"),
            file(&path, "m1_create_shop_table.rs", "// second"),
        ],
        lib_modules: vec!["m1_create_shop_table".to_string()],
        ..Plan::default()
    };

    assert!(commit_plan(&plan, &path).is_err());

    assert_eq!(fs::read_to_string(lib_path(&path)).unwrap(), before);
    assert_eq!(dir.entries(), ["src", "src/lib.rs"]);
}

#[test]
fn a_plan_without_a_module_creates_no_lib() {
    let dir = TempDir::new();
    let path = migrations_src(&dir, None);
    let plan = Plan {
        files: vec![file(&path, "snapshots/shop.rs", "// snapshot")],
        lib_modules: vec![],
        ..Plan::default()
    };

    commit_plan(&plan, &path).unwrap();

    assert_eq!(
        dir.entries(),
        ["src", "src/snapshots", "src/snapshots/shop.rs"]
    );
}

// What `run` does when no model changed: the framework's migrations are still
// wired into a `lib.rs` that lacks them, and a `lib.rs` already right is not rewritten.
#[test]
fn an_empty_plan_only_puts_the_framework_migrations_first() {
    let dir = TempDir::new();
    let path = migrations_src(&dir, Some("use sea_orm_migration::prelude::*;\n"));

    commit_plan(&Plan::default(), &path).unwrap();
    let wired = fs::read_to_string(lib_path(&path)).unwrap();
    assert_eq!(wired, initial_lib());

    let written = fs::metadata(lib_path(&path)).unwrap().modified().unwrap();
    commit_plan(&Plan::default(), &path).unwrap();
    assert_eq!(
        fs::metadata(lib_path(&path)).unwrap().modified().unwrap(),
        written
    );
    assert_eq!(dir.entries(), ["src", "src/lib.rs"]);
}

// The `sea-orm-cli migrate init` placeholder goes once the plan is written,
// and stays when it is not: a failed run changes nothing at all.
#[test]
fn the_placeholder_file_goes_only_with_a_plan_that_is_written() {
    let dir = TempDir::new();
    let path = migrations_src(&dir, Some(&initial_lib()));
    let placeholder = format!("{path}/{SEA_ORM_CLI_PLACEHOLDER_MODULE}.rs");
    fs::write(&placeholder, "// todo!()").unwrap();
    let twice = Plan {
        files: vec![
            file(&path, "m1_create_shop_table.rs", "// first"),
            file(&path, "m1_create_shop_table.rs", "// second"),
        ],
        lib_modules: vec![],
        ..Plan::default()
    };

    assert!(commit_plan(&twice, &path).is_err());
    assert!(Path::new(&placeholder).exists());

    commit_plan(&Plan::default(), &path).unwrap();
    assert!(!Path::new(&placeholder).exists());
}

// The whole command, on a project whose `lib.rs` can't be read (a directory
// by that name): it fails before writing a single file.
#[test]
fn a_run_that_cannot_read_lib_writes_no_file() {
    let dir = TempDir::new();
    let entities = dir.join("entities");
    fs::create_dir(&entities).unwrap();
    let path = migrations_src(&dir, None);
    fs::create_dir(lib_path(&path)).unwrap();

    assert!(run(entities.to_str().unwrap(), &path, false).is_err());

    let files: Vec<String> = dir
        .entries()
        .into_iter()
        .filter(|entry| dir.join(entry).is_file())
        .collect();
    assert!(files.is_empty(), "{files:?}");
}

// The power went in the middle of a run: `lib.rs` taken away, its content in
// the backup, one migration file placed. The next run takes all of it back
// before it reads anything, then finds nothing to do.
#[test]
fn a_run_cut_short_is_taken_back_before_the_next_one() {
    let dir = TempDir::new();
    let entities = dir.join("entities");
    fs::create_dir(&entities).unwrap();
    let before = initial_lib();
    let path = migrations_src(&dir, None);
    fs::write(format!("{path}/lib.rs.runique-bak"), &before).unwrap();
    fs::write(format!("{path}/lib.rs.runique-tmp"), "// half done").unwrap();
    fs::write(format!("{path}/m1_create_shop_table.rs"), "// shop").unwrap();
    fs::write(
        format!("{path}/m1_create_shop_table.rs.runique-tmp"),
        "// shop",
    )
    .unwrap();
    crate::atomic::tests::cut_journal(
        Path::new(&path),
        false,
        &[("m1_create_shop_table.rs", false), ("lib.rs", true)],
        &[],
    );

    run(entities.to_str().unwrap(), &path, false).unwrap();

    assert_eq!(fs::read_to_string(lib_path(&path)).unwrap(), before);
    assert!(!Path::new(&format!("{path}/m1_create_shop_table.rs")).exists());
    let leftovers: Vec<String> = dir
        .entries()
        .into_iter()
        .filter(|entry| entry.contains(".runique-"))
        .collect();
    assert!(leftovers.is_empty(), "{leftovers:?}");
}

// ── A run cut short, and a developer who came by before the next one ─────────

/// A run interrupted once `lib.rs` was replaced and one migration placed.
/// Returns the entities folder, the migrations path and the original `lib.rs`.
fn cut_run(dir: &TempDir) -> (String, String, String) {
    let entities = dir.join("entities");
    fs::create_dir(&entities).unwrap();
    let before = initial_lib();
    let path = migrations_src(dir, Some("// lib.rs with the new migration"));
    fs::write(
        format!("{path}/lib.rs.runique-tmp"),
        "// lib.rs with the new migration",
    )
    .unwrap();
    fs::write(format!("{path}/lib.rs.runique-bak"), &before).unwrap();
    fs::write(format!("{path}/m1_create_shop_table.rs"), "// shop").unwrap();
    fs::write(
        format!("{path}/m1_create_shop_table.rs.runique-tmp"),
        "// shop",
    )
    .unwrap();
    crate::atomic::tests::cut_journal(
        Path::new(&path),
        false,
        &[("m1_create_shop_table.rs", false), ("lib.rs", true)],
        &[],
    );
    (entities.to_str().unwrap().to_string(), path, before)
}

fn leftovers(dir: &TempDir) -> Vec<String> {
    dir.entries()
        .into_iter()
        .filter(|entry| entry.contains(".runique-"))
        .collect()
}

// Before the next run, the developer wrote a migration of their own, under a
// name the journal does not know: the run is taken back around it.
#[test]
fn a_migration_written_by_hand_before_the_repair_is_left_alone() {
    let dir = TempDir::new();
    let (entities, path, before) = cut_run(&dir);
    fs::write(format!("{path}/m0_by_hand.rs"), "// the developer's").unwrap();

    run(&entities, &path, false).unwrap();

    assert_eq!(fs::read_to_string(lib_path(&path)).unwrap(), before);
    assert_eq!(
        fs::read_to_string(format!("{path}/m0_by_hand.rs")).unwrap(),
        "// the developer's"
    );
    assert!(!Path::new(&format!("{path}/m1_create_shop_table.rs")).exists());
    assert!(leftovers(&dir).is_empty());
}

// The developer edited `lib.rs` after the cut: it is neither what the run was
// writing nor what the backup holds. The command stops, names it, and has
// changed nothing: choosing between the two is the developer's call.
#[test]
fn a_lib_edited_by_hand_after_the_cut_stops_the_command() {
    let dir = TempDir::new();
    let (entities, path, before) = cut_run(&dir);
    fs::write(lib_path(&path), "// edited by hand").unwrap();
    let on_disk = dir.entries();

    let error = run(&entities, &path, false).unwrap_err();

    assert!(format!("{error:#}").contains("lib.rs"), "{error:#}");
    assert_eq!(
        fs::read_to_string(lib_path(&path)).unwrap(),
        "// edited by hand"
    );
    assert_eq!(
        fs::read_to_string(format!("{path}/lib.rs.runique-bak")).unwrap(),
        before
    );
    assert!(Path::new(&format!("{path}/m1_create_shop_table.rs")).exists());
    assert_eq!(dir.entries(), on_disk);
}

// A first repair stopped on the edited `lib.rs`; the journal is still there.
// The developer drops their edit and writes a migration by hand, then runs the
// command again: the repair starts over, and goes around the new file.
#[test]
fn a_file_added_between_two_repairs_is_left_alone() {
    let dir = TempDir::new();
    let (entities, path, before) = cut_run(&dir);
    fs::write(lib_path(&path), "// edited by hand").unwrap();
    assert!(run(&entities, &path, false).is_err());
    assert!(Path::new(&format!("{path}/.runique-journal")).exists());

    fs::remove_file(lib_path(&path)).unwrap();
    fs::write(format!("{path}/m0_by_hand.rs"), "// the developer's").unwrap();
    run(&entities, &path, false).unwrap();

    assert_eq!(fs::read_to_string(lib_path(&path)).unwrap(), before);
    assert_eq!(
        fs::read_to_string(format!("{path}/m0_by_hand.rs")).unwrap(),
        "// the developer's"
    );
    assert!(!Path::new(&format!("{path}/m1_create_shop_table.rs")).exists());
    assert!(leftovers(&dir).is_empty());
}

// The developer put a file in the snapshots folder the cut run had created:
// that folder can't be taken away with the rest, so nothing is.
#[test]
fn a_file_added_in_a_folder_the_run_created_stops_the_command() {
    let dir = TempDir::new();
    let entities = dir.join("entities");
    fs::create_dir(&entities).unwrap();
    let path = migrations_src(&dir, Some(&initial_lib()));
    fs::create_dir(format!("{path}/snapshots")).unwrap();
    fs::write(
        format!("{path}/snapshots/shop.rs.runique-tmp"),
        "// snapshot",
    )
    .unwrap();
    fs::write(format!("{path}/snapshots/notes.md"), "the developer's").unwrap();
    crate::atomic::tests::cut_journal(
        Path::new(&path),
        false,
        &[("snapshots/shop.rs", false)],
        &["snapshots"],
    );
    let on_disk = dir.entries();

    let error = run(entities.to_str().unwrap(), &path, false).unwrap_err();

    assert!(format!("{error:#}").contains("notes.md"), "{error:#}");
    assert_eq!(dir.entries(), on_disk);
}

// ── `makemigrations`: the database first ─────────────────────────────────────

#[cfg(feature = "sqlite")]
#[tokio::test]
async fn with_a_reachable_database_the_command_runs() {
    let (entities, migrations) = (temp("ent_db_ok"), temp("mig_db_ok"));
    let get = |key: &str| (key == "DATABASE_URL").then(|| "sqlite::memory:".to_string());

    let result = makemigrations_with(
        get,
        entities.to_str().unwrap(),
        migrations.to_str().unwrap(),
        false,
    )
    .await;
    // `run` creates the snapshot folder first: proof the command went on.
    let reached = Path::new(&snapshot_dir(migrations.to_str().unwrap())).exists();
    let _ = (
        fs::remove_dir_all(&entities),
        fs::remove_dir_all(&migrations),
    );

    result.unwrap();
    assert!(reached);
}

/// Nothing listens on port 1: the command stops before `run`, writes nothing,
/// and its message never carries the password.
#[tokio::test]
async fn with_an_unreachable_database_nothing_is_written() {
    let (entities, migrations) = (temp("ent_db_ko"), temp("mig_db_ko"));
    let get = |key: &str| {
        (key == "DATABASE_URL")
            .then(|| "postgres://app:s3cr3t-Pw@127.0.0.1:1/shop?password=0ther-Pw".to_string())
    };

    let error = makemigrations_with(
        get,
        entities.to_str().unwrap(),
        migrations.to_str().unwrap(),
        false,
    )
    .await
    .unwrap_err();
    let written = fs::read_dir(&migrations).unwrap().count();
    let _ = (
        fs::remove_dir_all(&entities),
        fs::remove_dir_all(&migrations),
    );

    let message = format!("{error:#}");
    assert_eq!(written, 0, "{message}");
    assert!(!message.contains("s3cr3t-Pw"), "{message}");
    assert!(!message.contains("0ther-Pw"), "{message}");
}

// ── `snapshots/`: every snapshot is read before anything is computed ─────────

/// A `migration/src` whose `snapshots/` holds `files` (path under it, content).
fn with_snapshots(dir: &TempDir, files: &[(&str, &str)]) -> String {
    let path = migrations_src(dir, None);
    for (name, content) in files {
        let file = dir.join("src/snapshots").join(name);
        fs::create_dir_all(file.parent().unwrap()).unwrap();
        fs::write(file, content).unwrap();
    }
    path
}

#[test]
fn both_folders_are_read_and_the_model_tables_returned() {
    let dir = TempDir::new();
    let (menus, users) = (schema("menus", &[]), schema("eihwaz_users", &[]));
    let path = with_snapshots(
        &dir,
        &[
            ("menus.rs", &generate_snapshot_file(&menus)),
            ("runique/eihwaz_users.rs", &extend_snapshot_content(&users)),
        ],
    );

    let mut tables: Vec<String> = read_all(&path)
        .unwrap()
        .into_iter()
        .map(|s| s.table_name)
        .collect();
    tables.sort();
    // The extension is checked like the rest (see the refusals below), but
    // only the tables Runique migrated for a model come back.
    assert_eq!(tables, ["menus"]);
}

#[test]
fn without_a_snapshot_folder_nothing_is_read() {
    let dir = TempDir::new();
    assert!(read_all(&migrations_src(&dir, None)).unwrap().is_empty());
}

#[test]
fn a_file_that_is_not_rust_is_ignored() {
    let dir = TempDir::new();
    let path = with_snapshots(&dir, &[("notes.md", "not a snapshot"), ("menus.txt", "{")]);
    assert!(read_all(&path).unwrap().is_empty());
}

// Not Rust at all, or Rust that describes no table: the same refusal, in
// either folder, naming the file.
#[test]
fn an_unreadable_snapshot_is_refused_by_its_path() {
    for (name, content) in [
        ("menus.rs", "fn broken( {"),
        ("menus.rs", "fn main() {}"),
        ("runique/eihwaz_users.rs", "fn main() {}"),
    ] {
        let dir = TempDir::new();
        let path = with_snapshots(&dir, &[(name, content)]);
        let shown = dir.join("src/snapshots").join(name).display().to_string();

        assert_eq!(
            read_all(&path).unwrap_err().to_string(),
            tf("makemigrations.snapshot_unreadable", &[shown]),
            "{name}: {content}"
        );
    }
}

#[test]
fn a_snapshot_named_after_another_table_is_refused() {
    let dir = TempDir::new();
    let path = with_snapshots(
        &dir,
        &[("menus.rs", &generate_snapshot_file(&schema("plats", &[])))],
    );
    let shown = dir.join("src/snapshots/menus.rs").display().to_string();

    assert_eq!(
        read_all(&path).unwrap_err().to_string(),
        tf(
            "makemigrations.snapshot_misnamed",
            &[shown.as_str(), "plats"]
        )
    );
}

// The whole command: a bad snapshot of a table no model describes stops it
// before anything is written.
#[test]
fn the_command_stops_on_a_bad_snapshot_and_writes_nothing() {
    let dir = TempDir::new();
    let path = with_snapshots(&dir, &[("gone.rs", "fn main() {}")]);
    let entities = dir.join("entities");
    fs::create_dir_all(&entities).unwrap();
    let on_disk = dir.entries();

    assert!(run(entities.to_str().unwrap(), &path, false).is_err());
    assert_eq!(dir.entries(), on_disk);
}

// ── Run again before anything is applied: the waiting migrations are replaced ─

const USERS: &str = r#"model! { User, table: "users", pk: id => i32, { username: text [required], email: text [required], } }"#;
const USERS_BIO: &str = r#"model! { User, table: "users", pk: id => i32, { username: text [required], email: text [required], bio: text [nullable], } }"#;
const USERS_NO_EMAIL: &str = r#"model! { User, table: "users", pk: id => i32, { username: text [required], bio: text [nullable], } }"#;
const POSTS: &str =
    r#"model! { Post, table: "posts", pk: id => i32, { title: text [required], } }"#;

/// A project: its entities folder and its `migration/src`, both empty.
fn project(dir: &TempDir) -> (String, String) {
    let entities = dir.join("entities");
    fs::create_dir(&entities).unwrap();
    (
        entities.to_str().unwrap().to_string(),
        migrations_src(dir, None),
    )
}

/// The generated migrations under `path`: file name, content.
fn generated(path: &str) -> Vec<(String, String)> {
    let mut found: Vec<(String, String)> = fs::read_dir(path)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|file| {
            let name = file.file_name().unwrap().to_string_lossy().to_string();
            name.starts_with('m') && name.ends_with("_table.rs")
        })
        .map(|file| {
            (
                file.file_name().unwrap().to_string_lossy().to_string(),
                fs::read_to_string(&file).unwrap(),
            )
        })
        .collect();
    found.sort();
    found
}

/// Every file of `dir`, with its content: what a refusal must leave as it is.
fn everything(dir: &TempDir) -> Vec<(String, String)> {
    dir.entries()
        .into_iter()
        .filter(|entry| dir.join(entry).is_file())
        .map(|entry| {
            let content = fs::read_to_string(dir.join(&entry)).unwrap();
            (entry, content)
        })
        .collect()
}

/// What `migration up` does once the database took the migrations.
fn applied(entities: &str, path: &str) {
    super::migrate::record_applied(entities, path).unwrap();
}

// The command writes the migration and registers it; the snapshot waits for
// `migration up`.
#[test]
fn a_run_writes_no_snapshot() {
    let dir = TempDir::new();
    let (entities, path) = project(&dir);
    fs::write(format!("{entities}/user.rs"), USERS).unwrap();

    run(&entities, &path, false).unwrap();

    let files = generated(&path);
    assert_eq!(files.len(), 1, "{files:?}");
    assert!(files[0].0.ends_with("_create_users_table.rs"));
    assert!(has_marker(&files[0].1, WAIT_MIGRATION));
    assert!(!Path::new(&snapshot_file_path(&path, "users")).exists());

    applied(&entities, &path);
    assert!(Path::new(&snapshot_file_path(&path, "users")).exists());
}

// Nothing changed since the last run: not a single file is rewritten.
#[test]
fn a_run_again_without_a_change_rewrites_nothing() {
    let dir = TempDir::new();
    let (entities, path) = project(&dir);
    fs::write(format!("{entities}/user.rs"), USERS).unwrap();
    fs::write(format!("{entities}/post.rs"), POSTS).unwrap();
    run(&entities, &path, false).unwrap();
    let before = everything(&dir);

    run(&entities, &path, false).unwrap();

    assert_eq!(everything(&dir), before);
    assert_eq!(generated(&path).len(), 2);
}

// The model changed before the first migration was applied: the creation is
// computed again from nothing, whole. No alteration is piled on it.
#[test]
fn a_change_before_applying_replaces_the_waiting_creation() {
    let dir = TempDir::new();
    let (entities, path) = project(&dir);
    fs::write(format!("{entities}/user.rs"), USERS).unwrap();
    run(&entities, &path, false).unwrap();

    fs::write(format!("{entities}/user.rs"), USERS_BIO).unwrap();
    run(&entities, &path, false).unwrap();

    let files = generated(&path);
    assert_eq!(files.len(), 1, "{files:?}");
    assert!(files[0].0.ends_with("_create_users_table.rs"), "{files:?}");
    assert!(files[0].1.contains("bio"), "{}", files[0].1);
    let lib = fs::read_to_string(lib_path(&path)).unwrap();
    assert_eq!(lib.matches("_create_users_table").count(), 2, "{lib}");
    assert!(!lib.contains("_alter_"), "{lib}");
}

// The model the waiting creation was for is gone, another one came: the
// waiting file is removed with its line in `lib.rs`, the new one registered.
#[test]
fn a_waiting_migration_of_a_model_that_left_is_removed() {
    let dir = TempDir::new();
    let (entities, path) = project(&dir);
    fs::write(format!("{entities}/user.rs"), USERS).unwrap();
    run(&entities, &path, false).unwrap();

    fs::remove_file(format!("{entities}/user.rs")).unwrap();
    fs::write(format!("{entities}/post.rs"), POSTS).unwrap();
    run(&entities, &path, false).unwrap();

    let files = generated(&path);
    assert_eq!(files.len(), 1, "{files:?}");
    assert!(files[0].0.ends_with("_create_posts_table.rs"), "{files:?}");
    let lib = fs::read_to_string(lib_path(&path)).unwrap();
    assert!(!lib.contains("users_table"), "{lib}");
    assert_eq!(lib.matches("_create_posts_table").count(), 2, "{lib}");
    assert!(!crate::atomic::interrupted(Path::new(&path)));
}

// The change is taken back before it was applied: nothing is left to
// migrate, and the waiting migration goes. `lib.rs` is as at the start.
#[test]
fn a_change_taken_back_removes_the_waiting_migration() {
    let dir = TempDir::new();
    let (entities, path) = project(&dir);
    fs::write(format!("{entities}/user.rs"), USERS).unwrap();
    run(&entities, &path, false).unwrap();
    applied(&entities, &path);
    let lib_applied = fs::read_to_string(lib_path(&path)).unwrap();
    let create = generated(&path);

    fs::write(format!("{entities}/user.rs"), USERS_BIO).unwrap();
    run(&entities, &path, false).unwrap();
    assert_eq!(generated(&path).len(), 2);

    fs::write(format!("{entities}/user.rs"), USERS).unwrap();
    run(&entities, &path, false).unwrap();

    // Only the applied creation is left, untouched.
    assert_eq!(generated(&path), create);
    assert_eq!(fs::read_to_string(lib_path(&path)).unwrap(), lib_applied);
}

// Once applied, a migration is never replaced: the next change is an
// alteration next to it, and changing again replaces that alteration only.
#[test]
fn an_applied_migration_is_never_replaced() {
    let dir = TempDir::new();
    let (entities, path) = project(&dir);
    fs::write(format!("{entities}/user.rs"), USERS).unwrap();
    run(&entities, &path, false).unwrap();
    applied(&entities, &path);
    let create = generated(&path).remove(0);

    fs::write(format!("{entities}/user.rs"), USERS_BIO).unwrap();
    run(&entities, &path, false).unwrap();
    fs::write(format!("{entities}/user.rs"), USERS_NO_EMAIL).unwrap();
    run(&entities, &path, true).unwrap();

    let files = generated(&path);
    assert_eq!(files.len(), 2, "{files:?}");
    assert!(files.contains(&create), "{files:?}");
    let alter = files
        .iter()
        .find(|(name, _)| name.ends_with("_alter_users_table.rs"))
        .unwrap();
    // One alteration, from the applied state to the model: both changes.
    assert!(
        alter.1.contains("bio") && alter.1.contains("drop_column"),
        "{}",
        alter.1
    );
}

// A run that is refused writes nothing: the migration waiting since the run
// before stays exactly as it was.
#[test]
fn a_refused_run_leaves_the_waiting_migration_as_it_was() {
    let dir = TempDir::new();
    let (entities, path) = project(&dir);
    fs::write(format!("{entities}/user.rs"), USERS).unwrap();
    run(&entities, &path, false).unwrap();
    applied(&entities, &path);
    fs::write(format!("{entities}/user.rs"), USERS_BIO).unwrap();
    run(&entities, &path, false).unwrap();
    let before = everything(&dir);

    fs::write(format!("{entities}/user.rs"), USERS_NO_EMAIL).unwrap();
    assert!(run(&entities, &path, false).is_err());

    fs::write(format!("{entities}/user.rs"), USERS_BIO).unwrap();
    assert_eq!(everything(&dir), before);
}

// A waiting migration edited by hand is computed again and overwritten.
#[test]
fn a_waiting_migration_edited_by_hand_is_overwritten() {
    let dir = TempDir::new();
    let (entities, path) = project(&dir);
    fs::write(format!("{entities}/user.rs"), USERS).unwrap();
    run(&entities, &path, false).unwrap();
    let (name, content) = generated(&path).remove(0);

    fs::write(format!("{path}/{name}"), format!("{content}// by hand\n")).unwrap();
    run(&entities, &path, false).unwrap();

    let files = generated(&path);
    assert_eq!(files.len(), 1, "{files:?}");
    assert_eq!(files[0].1, content);
}

// The plan takes the place of what waits: same tables and contents, nothing
// is rewritten; otherwise every waiting module is dropped, except one the
// plan writes again under the same name.
#[test]
fn a_plan_replaces_what_waits_unless_it_says_the_same() {
    let waiting = |entries: &[(&str, &str)]| -> Vec<(String, String)> {
        entries
            .iter()
            .map(|(m, c)| (m.to_string(), c.to_string()))
            .collect()
    };
    let plan = |entries: &[(&str, &str)]| Plan {
        files: entries
            .iter()
            .map(|(m, c)| (format!("src/{m}.rs"), c.to_string()))
            .collect(),
        lib_modules: entries.iter().map(|(m, _)| m.to_string()).collect(),
        ..Plan::default()
    };
    let (old_a, new_a) = (
        "m20260101_000000_create_a_table",
        "m20260202_000000_create_a_table",
    );
    let (old_b, new_b) = (
        "m20260101_000000_alter_b_table",
        "m20260202_000000_alter_b_table",
    );

    // The same, whatever the order and the timestamps.
    let mut same = plan(&[(new_b, "b"), (new_a, "a")]);
    assert!(!replace_waiting(
        &mut same,
        &waiting(&[(old_a, "a"), (old_b, "b")])
    ));
    assert!(same.files.is_empty() && same.lib_modules.is_empty());
    assert!(same.dropped_modules.is_empty());

    // Each of these differs: a content, a table, one more, one less, a pair.
    for (planned, waits) in [
        (
            vec![(new_a, "a2"), (new_b, "b")],
            vec![(old_a, "a"), (old_b, "b")],
        ),
        (vec![(new_a, "a")], vec![(old_b, "a")]),
        (vec![(new_a, "a"), (new_b, "b")], vec![(old_a, "a")]),
        (vec![(new_a, "a")], vec![(old_a, "a"), (old_b, "b")]),
        (
            vec![(new_a, "a"), (new_b, "b")],
            vec![(old_a, "a"), (old_a, "a")],
        ),
    ] {
        let mut changed = plan(&planned);
        let waits = waiting(&waits);
        assert!(
            replace_waiting(&mut changed, &waits),
            "{planned:?} / {waits:?}"
        );
        assert_eq!(changed.files.len(), planned.len());
        // None of these plans reuses a waiting name: every one is dropped.
        let every: Vec<String> = waits.iter().map(|(m, _)| m.clone()).collect();
        assert_eq!(changed.dropped_modules, every);
    }

    // Nothing waits: the plan is written as it is.
    let mut first = plan(&[(new_a, "a")]);
    assert!(replace_waiting(&mut first, &[]));
    assert_eq!(first.files.len(), 1);
    assert!(first.dropped_modules.is_empty());

    // Written again under the very same name: overwritten, not removed.
    let mut again = plan(&[(old_a, "a2")]);
    assert!(replace_waiting(
        &mut again,
        &waiting(&[(old_a, "a"), (old_b, "b")])
    ));
    assert_eq!(again.dropped_modules, [old_b]);
}

// The plan's dropped modules leave the folder and `lib.rs` in the same batch.
#[test]
fn a_plan_removes_the_modules_it_drops() {
    let dir = TempDir::new();
    let path = migrations_src(&dir, Some(&initial_lib()));
    let first = Plan {
        files: vec![
            file(&path, "m1_create_shop_table.rs", "// shop"),
            file(&path, "m1_create_menu_table.rs", "// menu"),
        ],
        lib_modules: vec![
            "m1_create_shop_table".to_string(),
            "m1_create_menu_table".to_string(),
        ],
        ..Plan::default()
    };
    commit_plan(&first, &path).unwrap();

    let second = Plan {
        files: vec![file(&path, "m2_create_shop_table.rs", "// shop, again")],
        lib_modules: vec!["m2_create_shop_table".to_string()],
        dropped_modules: vec!["m1_create_shop_table".to_string()],
    };
    commit_plan(&second, &path).unwrap();

    assert_eq!(
        dir.entries(),
        [
            "src",
            "src/lib.rs",
            "src/m1_create_menu_table.rs",
            "src/m2_create_shop_table.rs",
        ]
    );
    let lib = fs::read_to_string(lib_path(&path)).unwrap();
    assert!(!lib.contains("m1_create_shop_table"), "{lib}");
    assert_eq!(lib.matches("m2_create_shop_table").count(), 2, "{lib}");
    assert_eq!(lib.matches("m1_create_menu_table").count(), 2, "{lib}");
}

// ── Each migration says where it starts from; two from the same point clash ──

#[test]
fn a_migration_says_what_it_starts_from() {
    assert_eq!(started_from("// runique: from none\nfn up() {}"), Some("none"));
    assert_eq!(
        started_from("// tag\n  // runique: from m1_create_a_table \r\nfn up() {}"),
        Some("m1_create_a_table")
    );
    assert_eq!(started_from("fn up() {}\n"), None);
    assert_eq!(started_from("// see runique: from m1\n"), None);
}

// The line sits under the tag: `none` for a table nothing settled touches,
// otherwise the last settled migration of that very table and kind.
#[test]
fn a_planned_migration_starts_from_the_last_settled_one_of_its_table() {
    let (menus, plats, users) = (
        schema("menus", &[]),
        schema("plats", &[]),
        schema("eihwaz_users", &[]),
    );
    let altered = |s: &ParsedSchema| Changes {
        is_new_table: false,
        ..new_table(s)
    };
    let settled: Vec<String> = [
        "m20250101_000000_create_plats_table",
        "m20250102_000000_alter_plats_table",
        "m20250103_000000_create_other_table",
        // A model's table of that name is not the extension of it.
        "m20250104_000000_create_eihwaz_users_table",
    ]
    .map(String::from)
    .to_vec();

    let mut plan = Plan::default();
    build_main_plan(
        &mut plan,
        &[new_table(&menus), altered(&plats)],
        &[menus.clone(), plats.clone()],
        "m",
        "20260101_000000",
        &settled,
    );
    build_extend_plan(
        &mut plan,
        &[(users.clone(), altered(&users))],
        "m",
        "20260101_000000",
        &settled,
    );

    let from: Vec<Option<&str>> = plan.files.iter().map(|(_, c)| started_from(c)).collect();
    assert_eq!(
        from,
        [
            Some("none"),
            Some("m20250102_000000_alter_plats_table"),
            Some("none"),
        ]
    );
    for (path, content) in &plan.files {
        let second = content.lines().nth(1).unwrap();
        assert!(second.starts_with(FROM_MIGRATION), "{path}: {second}");
    }
}

/// A `migration/src` whose `lib.rs` declares `files` (module, started from).
fn with_migrations(dir: &TempDir, files: &[(&str, Option<&str>)]) -> String {
    let mods: String = files.iter().map(|(m, _)| format!("mod {m};\n")).collect();
    let path = migrations_src(dir, Some(&mods));
    for (module, from) in files {
        let line = from.map_or(String::new(), |f| format!("{FROM_MIGRATION}{f}\n"));
        fs::write(format!("{path}/{module}.rs"), format!("{line}// body\n")).unwrap();
    }
    path
}

// Two migrations of a table from the same point: refused, naming the table
// and both. That includes two creations of the same table.
#[test]
fn two_migrations_of_a_table_from_the_same_point_are_refused() {
    let (create, a, b) = (
        "m20260101_000000_create_menus_table",
        "m20260102_000000_alter_menus_table",
        "m20260103_000000_alter_menus_table",
    );
    let dir = TempDir::new();
    let path = with_migrations(
        &dir,
        &[(create, Some("none")), (a, Some(create)), (b, Some(create))],
    );
    assert_eq!(
        check_parallel(&path).unwrap_err().to_string(),
        tf(
            "makemigrations.parallel_changes",
            &[format!("menus: {a}, {b}")]
        )
    );

    let other = "m20260105_000000_create_menus_table";
    let dir = TempDir::new();
    let path = with_migrations(&dir, &[(create, Some("none")), (other, Some("none"))]);
    let error = check_parallel(&path).unwrap_err().to_string();
    assert!(error.contains(create) && error.contains(other), "{error}");
}

// A chain, two tables from `none`, a model's table and the extension of the
// same name, migrations that say nothing, a module without a file: all pass.
#[test]
fn migrations_that_follow_each_other_pass() {
    let dir = TempDir::new();
    let path = with_migrations(
        &dir,
        &[
            ("m20260101_000000_create_menus_table", Some("none")),
            ("m20260101_000000_create_plats_table", Some("none")),
            (
                "m20260102_000000_alter_menus_table",
                Some("m20260101_000000_create_menus_table"),
            ),
            (
                "m20260103_000000_alter_menus_table",
                Some("m20260102_000000_alter_menus_table"),
            ),
            ("m20260104_000000_create_eihwaz_users_table", Some("none")),
            ("m20260104_000000_extend_eihwaz_users_table", Some("none")),
            ("m20250101_000000_alter_menus_table", None),
            ("m20250102_000000_alter_menus_table", None),
            ("m0_by_hand", Some("none")),
            ("m0_by_hand_too", Some("none")),
        ],
    );
    fs::write(
        lib_path(&path),
        format!(
            "{}mod m20260109_000000_alter_menus_table;\n",
            fs::read_to_string(lib_path(&path)).unwrap()
        ),
    )
    .unwrap();

    check_parallel(&path).unwrap();
}

// The whole command: an alteration starts from the applied creation, running
// again keeps that line, and the next one starts from the applied alteration.
#[test]
fn the_command_chains_the_migrations_of_a_table() {
    let dir = TempDir::new();
    let (entities, path) = project(&dir);
    fs::write(format!("{entities}/user.rs"), USERS).unwrap();
    run(&entities, &path, false).unwrap();
    let create = generated(&path).remove(0);
    assert_eq!(started_from(&create.1), Some("none"));
    applied(&entities, &path);

    fs::write(format!("{entities}/user.rs"), USERS_BIO).unwrap();
    run(&entities, &path, false).unwrap();
    run(&entities, &path, false).unwrap();
    let files = generated(&path);
    let alter = files
        .iter()
        .find(|(name, _)| name.ends_with("_alter_users_table.rs"))
        .unwrap();
    assert_eq!(
        started_from(&alter.1),
        Some(create.0.trim_end_matches(".rs"))
    );
    // Applied: the tag goes, the line stays for the next check.
    applied(&entities, &path);
    let alter = generated(&path)
        .into_iter()
        .find(|(name, _)| name.ends_with("_alter_users_table.rs"))
        .unwrap();
    assert!(!has_marker(&alter.1, WAIT_MIGRATION));
    assert_eq!(
        started_from(&alter.1),
        Some(create.0.trim_end_matches(".rs"))
    );
}

// The whole command on a project holding two parallel alterations: it stops
// before anything is computed, and writes nothing.
#[test]
fn the_command_refuses_parallel_migrations_and_writes_nothing() {
    let dir = TempDir::new();
    let (entities, path) = project(&dir);
    fs::write(format!("{entities}/user.rs"), USERS).unwrap();
    run(&entities, &path, false).unwrap();
    applied(&entities, &path);
    let create = generated(&path).remove(0).0.trim_end_matches(".rs").to_string();
    for module in [
        "m20990101_000000_alter_users_table",
        "m20990102_000000_alter_users_table",
    ] {
        fs::write(
            format!("{path}/{module}.rs"),
            format!("{FROM_MIGRATION}{create}\n// body\n"),
        )
        .unwrap();
        let lib = fs::read_to_string(lib_path(&path)).unwrap();
        fs::write(lib_path(&path), format!("{lib}mod {module};\n")).unwrap();
    }
    fs::write(format!("{entities}/user.rs"), USERS_BIO).unwrap();
    let before = everything(&dir);

    let error = run(&entities, &path, false).unwrap_err().to_string();

    assert!(
        error.contains(
            "users: m20990101_000000_alter_users_table, m20990102_000000_alter_users_table"
        ),
        "{error}"
    );
    assert_eq!(everything(&dir), before);
}

// ── A table Runique migrated and that no model produces any more ─────────────

const CATEGORIES: &str =
    r#"model! { Categorie, table: "categories", pk: id => i32, { nom: text [required], } }"#;
const MENUS_IN_CATEGORY: &str = r#"model! { Menu, table: "menus", pk: id => i32, { titre: text [required], categorie_id: int [nullable] },
    relations: { belongs_to: categorie via categorie_id [set_null] } }"#;
// The same table once the relation is given up: the column stays, the key goes.
const MENUS_ALONE: &str = r#"model! { Menu, table: "menus", pk: id => i32, { titre: text [required], categorie_id: int [nullable] } }"#;

/// The names of the generated migrations under `path`, in the order `lib.rs`
/// runs them.
fn run_order(path: &str) -> Vec<String> {
    let lib = fs::read_to_string(lib_path(path)).unwrap();
    let mut modules: Vec<(usize, String)> = generated(path)
        .into_iter()
        .map(|(name, _)| name.trim_end_matches(".rs").to_string())
        .filter_map(|module| {
            lib.find(&format!("Box::new({module}::Migration)"))
                .map(|at| (at, module))
        })
        .collect();
    modules.sort();
    modules.into_iter().map(|(_, module)| module).collect()
}

fn kind_of(module: &str) -> &str {
    // `m{date}_{time}_{kind}_{table}_table`
    module.splitn(3, '_').nth(2).unwrap_or_default()
}

// The snapshots of `snapshots/` without a model are the tables to drop, by
// name; a table of the framework never is, and a table with a model neither.
#[test]
fn a_migrated_table_without_a_model_is_to_drop() {
    let snapshots = || {
        vec![
            schema("plats", &[]),
            schema("menus", &[]),
            schema("eihwaz_users", &[]),
            schema("eihwaz_custom", &[]),
            schema("categories", &[]),
        ]
    };
    let names = |dropped: Vec<ParsedSchema>| -> Vec<String> {
        dropped.into_iter().map(|s| s.table_name).collect()
    };

    assert_eq!(
        names(dropped_tables(snapshots(), &[schema("menus", &[])])),
        ["categories", "plats"]
    );
    assert_eq!(
        names(dropped_tables(snapshots(), &[])),
        ["categories", "menus", "plats"]
    );
    assert!(dropped_tables(vec![], &[schema("menus", &[])]).is_empty());
}

// One line per table, the wording the guard shows.
#[test]
fn a_dropped_table_is_announced_as_a_loss_of_data() {
    assert_eq!(
        dropped_table_messages(&["menus".to_string(), "plats".to_string()]),
        ["  menus: DROP TABLE (data loss)", "  plats: DROP TABLE (data loss)"]
    );
    assert!(dropped_table_messages(&[]).is_empty());
}

// The table that references goes before the one it references, whatever
// their names; a table on its own keeps its place by name.
#[test]
fn tables_are_dropped_from_the_one_that_references() {
    let order = |tables: Vec<ParsedSchema>| -> Vec<String> {
        let mut plan = Plan::default();
        build_drop_plan(&mut plan, &tables, "m", "20260101_000000", &[]);
        plan.lib_modules
            .iter()
            .map(|m| migration_target(m).unwrap().1.to_string())
            .collect()
    };

    assert_eq!(
        order(vec![schema("a_parent", &[]), schema("z_child", &["a_parent"])]),
        ["z_child", "a_parent"]
    );
    assert_eq!(
        order(vec![schema("z_parent", &[]), schema("a_child", &["z_parent"])]),
        ["a_child", "z_parent"]
    );
    // A chain: the furthest from the root first.
    assert_eq!(
        order(vec![
            schema("b", &["a"]),
            schema("a", &[]),
            schema("c", &["b"]),
        ]),
        ["c", "b", "a"]
    );
    // A key to a table that stays, or to itself, asks for no order.
    assert_eq!(
        order(vec![schema("b", &["kept"]), schema("a", &["a"])]),
        ["b", "a"]
    );
}

// The file that drops a table: `IF EXISTS` in `up`, the table created again
// in `down`, the tag and the starting point on top like any migration.
#[test]
fn a_drop_migration_mirrors_the_creation() {
    let menus = schema("menus", &["categories"]);
    let mut plan = Plan::default();
    build_drop_plan(
        &mut plan,
        std::slice::from_ref(&menus),
        "m",
        "20260101_000000",
        &["m20250101_000000_create_menus_table".to_string()],
    );

    assert_eq!(plan.lib_modules, ["m20260101_000000_drop_menus_table"]);
    let (path, content) = &plan.files[0];
    assert_eq!(path, "m/m20260101_000000_drop_menus_table.rs");
    assert!(content.starts_with(&format!("{WAIT_MIGRATION}\n")), "{content}");
    assert_eq!(
        started_from(content),
        Some("m20250101_000000_create_menus_table")
    );
    let (up, down) = content.split_once("async fn down").unwrap();
    assert!(up.contains(".drop_table(Table::drop()"), "{up}");
    assert!(up.contains(".if_exists()"), "{up}");
    assert!(!up.contains("create_table"), "{up}");
    assert!(down.contains("Table::create()"), "{down}");
    assert!(down.contains("menus_categories_id_categories_fkey"), "{down}");
    assert!(!down.contains("drop_table"), "{down}");
    // The creation it mirrors drops without `IF EXISTS`, as before.
    assert!(!generate_create_file(&menus).contains(".if_exists()"));
}

// A cycle of keys between two dropped tables: no order works, so the key
// still pointing to the first table to go is removed before it, off SQLite.
#[test]
fn a_cycle_of_dropped_tables_loses_its_closing_key_first() {
    let mut plan = Plan::default();
    build_drop_plan(
        &mut plan,
        &[schema("a", &["b"]), schema("b", &["a"])],
        "m",
        "20260101_000000",
        &[],
    );

    // Created `a` then `b`: dropped `b` first, which `a` still points to.
    assert_eq!(
        plan.lib_modules,
        [
            "m20260101_000000_drop_b_table",
            "m20260101_000000_drop_a_table"
        ]
    );
    let (up_b, _) = plan.files[0].1.split_once("async fn down").unwrap();
    let key = up_b.find("a_b_id_b_fkey").expect("the closing key");
    let table = up_b.find(".drop_table(").expect("the drop");
    assert!(key < table, "{up_b}");
    assert!(up_b.contains("DbBackend::Sqlite"), "{up_b}");
    let (up_a, _) = plan.files[1].1.split_once("async fn down").unwrap();
    assert!(!up_a.contains("drop_foreign_key"), "{up_a}");
}

// The whole command: refused without `--force`, naming the table, and
// nothing is written, not even the harmless change of the same run.
#[test]
fn a_table_to_drop_is_refused_without_force() {
    let dir = TempDir::new();
    let (entities, path) = project(&dir);
    fs::write(format!("{entities}/user.rs"), USERS).unwrap();
    fs::write(format!("{entities}/post.rs"), POSTS).unwrap();
    run(&entities, &path, false).unwrap();
    applied(&entities, &path);

    fs::remove_file(format!("{entities}/post.rs")).unwrap();
    fs::write(format!("{entities}/user.rs"), USERS_BIO).unwrap();
    let before = everything(&dir);

    assert!(run(&entities, &path, false).is_err());
    assert_eq!(everything(&dir), before);
}

// With `--force`: the migration is written, the snapshot stays until the
// drop is applied, then goes. Running again in between rewrites nothing.
#[test]
fn a_forced_drop_keeps_the_snapshot_until_it_is_applied() {
    let dir = TempDir::new();
    let (entities, path) = project(&dir);
    fs::write(format!("{entities}/user.rs"), USERS).unwrap();
    fs::write(format!("{entities}/post.rs"), POSTS).unwrap();
    run(&entities, &path, false).unwrap();
    applied(&entities, &path);
    fs::remove_file(format!("{entities}/post.rs")).unwrap();

    run(&entities, &path, true).unwrap();
    let files = generated(&path);
    let drop = files
        .iter()
        .find(|(name, _)| name.ends_with("_drop_posts_table.rs"))
        .expect("the drop migration");
    assert!(has_marker(&drop.1, WAIT_MIGRATION));
    assert!(Path::new(&snapshot_file_path(&path, "posts")).exists());

    // Case 13: the same drop is computed again, not a file moves.
    let waiting = everything(&dir);
    run(&entities, &path, true).unwrap();
    assert_eq!(everything(&dir), waiting);

    applied(&entities, &path);
    assert!(!Path::new(&snapshot_file_path(&path, "posts")).exists());
    assert!(Path::new(&snapshot_file_path(&path, "users")).exists());
    assert!(!crate::atomic::interrupted(Path::new(&path)));

    // Applied and gone: nothing more to do, with or without `--force`.
    let settled = everything(&dir);
    run(&entities, &path, false).unwrap();
    assert_eq!(everything(&dir), settled);
}

// The table a `down` brings back has its columns in the order the model
// declared them, not in the order the snapshot reader returns them.
#[test]
fn a_dropped_table_comes_back_with_its_columns_in_order() {
    let dir = TempDir::new();
    let (entities, path) = project(&dir);
    fs::write(format!("{entities}/user.rs"), USERS_BIO).unwrap();
    run(&entities, &path, false).unwrap();
    applied(&entities, &path);
    let created = generated(&path).remove(0).1;
    fs::remove_file(format!("{entities}/user.rs")).unwrap();

    run(&entities, &path, true).unwrap();

    let drop = generated(&path)
        .into_iter()
        .find(|(name, _)| name.ends_with("_drop_users_table.rs"))
        .unwrap()
        .1;
    let (_, down) = drop.split_once("async fn down").unwrap();
    let order = |text: &str| -> Vec<usize> {
        ["\"username\"", "\"email\"", "\"bio\""]
            .iter()
            .map(|column| text.find(column).expect("the column"))
            .collect()
    };
    let (recreated, original) = (order(down), order(&created));
    assert!(original.is_sorted(), "{created}");
    assert!(recreated.is_sorted(), "{down}");
}

// Case 44: the model comes back before the drop is applied. The gap is nil,
// the waiting drop goes with its line in `lib.rs`.
#[test]
fn a_waiting_drop_goes_when_the_model_comes_back() {
    let dir = TempDir::new();
    let (entities, path) = project(&dir);
    fs::write(format!("{entities}/post.rs"), POSTS).unwrap();
    run(&entities, &path, false).unwrap();
    applied(&entities, &path);
    let settled = everything(&dir);

    fs::remove_file(format!("{entities}/post.rs")).unwrap();
    run(&entities, &path, true).unwrap();
    assert_eq!(generated(&path).len(), 2);

    fs::write(format!("{entities}/post.rs"), POSTS).unwrap();
    run(&entities, &path, false).unwrap();
    assert_eq!(everything(&dir), settled);
}

// Case 10: the last model of the project is removed. Same treatment, and
// `migration up` then records it without a single model left to read.
#[test]
fn the_last_table_of_a_project_is_dropped_like_any_other() {
    let dir = TempDir::new();
    let (entities, path) = project(&dir);
    fs::write(format!("{entities}/post.rs"), POSTS).unwrap();
    run(&entities, &path, false).unwrap();
    applied(&entities, &path);
    fs::remove_file(format!("{entities}/post.rs")).unwrap();

    assert!(run(&entities, &path, false).is_err());
    run(&entities, &path, true).unwrap();
    fs::remove_dir(&entities).unwrap();
    super::migrate::record_applied(&entities, &path).unwrap();

    assert!(!Path::new(&snapshot_file_path(&path, "posts")).exists());
}

// Case 27: a table that stays gave up its key to the dropped one. Its own
// migration removes the key first, the drop comes after it in `lib.rs`.
#[test]
fn the_key_of_a_table_that_stays_is_removed_before_the_drop() {
    let dir = TempDir::new();
    let (entities, path) = project(&dir);
    fs::write(format!("{entities}/categorie.rs"), CATEGORIES).unwrap();
    fs::write(format!("{entities}/menu.rs"), MENUS_IN_CATEGORY).unwrap();
    run(&entities, &path, false).unwrap();
    applied(&entities, &path);

    fs::remove_file(format!("{entities}/categorie.rs")).unwrap();
    fs::write(format!("{entities}/menu.rs"), MENUS_ALONE).unwrap();
    run(&entities, &path, true).unwrap();

    let order = run_order(&path);
    let waiting: Vec<&str> = order
        .iter()
        .map(String::as_str)
        .filter(|m| !kind_of(m).starts_with("create_"))
        .collect();
    assert_eq!(waiting.len(), 2, "{order:?}");
    assert!(waiting[0].ends_with("_alter_menus_table"), "{order:?}");
    assert!(waiting[1].ends_with("_drop_categories_table"), "{order:?}");
    let alter = generated(&path)
        .into_iter()
        .find(|(name, _)| name.ends_with("_alter_menus_table.rs"))
        .unwrap()
        .1;
    assert!(alter.contains("runtime::drop_foreign_key(manager, \"menus\""), "{alter}");
}

// Case 26: the model that stays still points to the removed one. Making the
// models agree is the developer's job: the command stops, `--force` or not.
#[test]
fn a_model_still_pointing_to_a_removed_one_stops_the_command() {
    let dir = TempDir::new();
    let (entities, path) = project(&dir);
    fs::write(format!("{entities}/categorie.rs"), CATEGORIES).unwrap();
    fs::write(format!("{entities}/menu.rs"), MENUS_IN_CATEGORY).unwrap();
    run(&entities, &path, false).unwrap();
    applied(&entities, &path);
    fs::remove_file(format!("{entities}/categorie.rs")).unwrap();
    let before = everything(&dir);

    for force in [false, true] {
        let error = run(&entities, &path, force).unwrap_err().to_string();
        assert!(error.contains("categorie"), "{error}");
        assert_eq!(everything(&dir), before);
    }
}
