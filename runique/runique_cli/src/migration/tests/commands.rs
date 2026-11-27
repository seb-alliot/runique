//! Written from cargo-mutants survivors (2026-10-08): the private steps of
//! `run` — snapshot upgrades, lib.rs editing, ordering, guards.
use super::*;
use crate::migration::steps::generate::lib_file::*;
use crate::migration::steps::generate::plan::*;
use crate::migration::steps::parser::files::snapshot::*;
use crate::migration::types::{ParsedColumn, ParsedFk};
use std::fs;
use std::path::Path;

/// Only a plain type change is listed: a change to or from an enum is
/// migrated, and a column whose other attributes changed isn't concerned.
#[test]
fn only_plain_type_changes_are_reported_as_unmigrated() {
    let col = |col_type: &str, enum_values: &[&str], nullable: bool| ParsedColumn {
        name: "c".into(),
        col_type: crate::migration::ColumnType::from_name(col_type)
            .expect("the name of a column type"),
        nullable,
        enum_string_values: enum_values.iter().map(|v| v.to_string()).collect(),
        ..ParsedColumn::default()
    };
    let changes = |old: ParsedColumn, new: ParsedColumn| Changes {
        table_name: "t".into(),
        modified_columns: vec![(old, new)],
        ..Default::default()
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
            col_type: crate::migration::ColumnType::Integer,
            ..ParsedColumn::default()
        }),
        columns: vec![ParsedColumn {
            name: "title".into(),
            col_type: crate::migration::ColumnType::String,
            max_length: Some(50),
            ..ParsedColumn::default()
        }],
        foreign_keys: fks
            .iter()
            .map(|to| ParsedFk {
                from_column: format!("{to}_id"),
                to_table: (*to).to_string(),
                to_column: "id".into(),
                on_delete: crate::migration::FkAction::Cascade,
                on_update: crate::migration::FkAction::NoAction,
            })
            .collect(),
        indexes: vec![],
    }
}

fn new_table(s: &ParsedSchema) -> Changes {
    Changes {
        table_name: s.table_name.clone(),
        added_columns: s.columns.clone(),
        added_fks: s.foreign_keys.clone(),
        table: crate::migration::TableChange::Created,
        ..Default::default()
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

    run(e, m, false, DatabaseEngine::SQLite).unwrap();
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
        on_delete: crate::migration::FkAction::Cascade,
        on_update: crate::migration::FkAction::NoAction,
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
        table: crate::migration::TableChange::Altered,
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

    assert!(
        run(
            entities.to_str().unwrap(),
            &path,
            false,
            DatabaseEngine::SQLite
        )
        .is_err()
    );

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

    run(
        entities.to_str().unwrap(),
        &path,
        false,
        DatabaseEngine::SQLite,
    )
    .unwrap();

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

    run(&entities, &path, false, DatabaseEngine::SQLite).unwrap();

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

    let error = run(&entities, &path, false, DatabaseEngine::SQLite).unwrap_err();

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
    assert!(run(&entities, &path, false, DatabaseEngine::SQLite).is_err());
    assert!(Path::new(&format!("{path}/.runique-journal")).exists());

    fs::remove_file(lib_path(&path)).unwrap();
    fs::write(format!("{path}/m0_by_hand.rs"), "// the developer's").unwrap();
    run(&entities, &path, false, DatabaseEngine::SQLite).unwrap();

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

    let error = run(
        entities.to_str().unwrap(),
        &path,
        false,
        DatabaseEngine::SQLite,
    )
    .unwrap_err();

    assert!(format!("{error:#}").contains("notes.md"), "{error:#}");
    assert_eq!(dir.entries(), on_disk);
}

// ── The steps of a run ───────────────────────────────────────────────────────

// The plan comes out in the order of the run, each file beside its module.
#[test]
fn the_steps_of_a_run_come_out_in_their_order() {
    let step = |name: &str| Plan {
        files: vec![(format!("{name}.rs"), name.to_string())],
        lib_modules: vec![name.to_string()],
        ..Plan::default()
    };
    let steps = Run {
        deleted: step("deleted"),
        enums_modified: step("enums_modified"),
        tables: step("tables"),
        enums_created: step("enums_created"),
    };

    let plan = steps.in_order();

    let order = ["enums_created", "tables", "enums_modified", "deleted"];
    assert_eq!(plan.lib_modules, order);
    let files: Vec<&str> = plan
        .files
        .iter()
        .map(|(_, content)| content.as_str())
        .collect();
    assert_eq!(files, order);
    assert!(plan.dropped_modules.is_empty());
}

// A step given several files keeps them as they were given.
#[test]
fn the_files_of_a_step_keep_the_order_they_were_given_in() {
    let mut steps = Run::default();
    for name in ["shop", "book", "author"] {
        steps
            .tables
            .files
            .push((format!("{name}.rs"), String::new()));
        steps.tables.lib_modules.push(name.to_string());
    }

    assert_eq!(steps.in_order().lib_modules, ["shop", "book", "author"]);
}

// ── `makemigrations`: the database first ─────────────────────────────────────

/// Cargo's description of a project whose `runique` has `features`, as the
/// command would get it from `cargo metadata`.
fn project_with(features: &'static [&'static str]) -> impl FnOnce() -> Result<String> {
    move || {
        Ok(serde_json::json!({
            "packages": [{ "name": "runique", "id": "runique 4.1.0" }],
            "resolve": { "nodes": [{ "id": "runique 4.1.0", "features": features }] },
        })
        .to_string())
    }
}

#[cfg(feature = "sqlite")]
#[tokio::test]
async fn with_a_reachable_database_the_command_runs() {
    let (entities, migrations) = (temp("ent_db_ok"), temp("mig_db_ok"));
    let get = |key: &str| (key == "DATABASE_URL").then(|| "sqlite::memory:".to_string());

    let result = makemigrations_with(
        get,
        project_with(&["orm", "sqlite"]),
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
        project_with(&["orm", "postgres"]),
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

/// `.env` asks for SQLite, the project is compiled for PostgreSQL: the command
/// stops before it connects, and writes nothing.
#[tokio::test]
async fn an_engine_the_project_is_not_compiled_for_stops_the_command() {
    let (entities, migrations) = (temp("ent_engine_ko"), temp("mig_engine_ko"));
    let get = |key: &str| (key == "DATABASE_URL").then(|| "sqlite::memory:".to_string());

    let error = makemigrations_with(
        get,
        project_with(&["orm", "postgres"]),
        entities.to_str().unwrap(),
        migrations.to_str().unwrap(),
        false,
    )
    .await
    .unwrap_err()
    .to_string();
    let written = fs::read_dir(&migrations).unwrap().count();
    let _ = (
        fs::remove_dir_all(&entities),
        fs::remove_dir_all(&migrations),
    );

    assert_eq!(
        error,
        tf(
            runique_i18n::keys::makemigrations::error::ENGINE_NOT_IN_FEATURES,
            &["SQLite", "postgres", "SQLite", "sqlite"]
        )
    );
    assert_eq!(written, 0);
}

/// Cargo can't describe the project: the command stops there, with cargo's
/// reason, and writes nothing.
#[tokio::test]
async fn without_cargo_s_answer_the_command_stops() {
    let (entities, migrations) = (temp("ent_cargo_ko"), temp("mig_cargo_ko"));
    let get = |key: &str| (key == "DATABASE_URL").then(|| "sqlite::memory:".to_string());

    let error = makemigrations_with(
        get,
        || anyhow::bail!("no Cargo.toml here"),
        entities.to_str().unwrap(),
        migrations.to_str().unwrap(),
        false,
    )
    .await
    .unwrap_err()
    .to_string();
    let written = fs::read_dir(&migrations).unwrap().count();
    let _ = (
        fs::remove_dir_all(&entities),
        fs::remove_dir_all(&migrations),
    );

    assert_eq!(error, "no Cargo.toml here");
    assert_eq!(written, 0);
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
            tf(
                runique_i18n::keys::makemigrations::error::SNAPSHOT_UNREADABLE,
                &[shown]
            ),
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
            runique_i18n::keys::makemigrations::error::SNAPSHOT_MISNAMED,
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

    assert!(
        run(
            entities.to_str().unwrap(),
            &path,
            false,
            DatabaseEngine::SQLite
        )
        .is_err()
    );
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
    crate::migration::steps::apply::migrate::record_applied(entities, path).unwrap();
}

// The command writes the migration and registers it; the snapshot waits for
// `migration up`.
#[test]
fn a_run_writes_no_snapshot() {
    let dir = TempDir::new();
    let (entities, path) = project(&dir);
    fs::write(format!("{entities}/user.rs"), USERS).unwrap();

    run(&entities, &path, false, DatabaseEngine::SQLite).unwrap();

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
    run(&entities, &path, false, DatabaseEngine::SQLite).unwrap();
    let before = everything(&dir);

    run(&entities, &path, false, DatabaseEngine::SQLite).unwrap();

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
    run(&entities, &path, false, DatabaseEngine::SQLite).unwrap();

    fs::write(format!("{entities}/user.rs"), USERS_BIO).unwrap();
    run(&entities, &path, false, DatabaseEngine::SQLite).unwrap();

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
    run(&entities, &path, false, DatabaseEngine::SQLite).unwrap();

    fs::remove_file(format!("{entities}/user.rs")).unwrap();
    fs::write(format!("{entities}/post.rs"), POSTS).unwrap();
    run(&entities, &path, false, DatabaseEngine::SQLite).unwrap();

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
    run(&entities, &path, false, DatabaseEngine::SQLite).unwrap();
    applied(&entities, &path);
    let lib_applied = fs::read_to_string(lib_path(&path)).unwrap();
    let create = generated(&path);

    fs::write(format!("{entities}/user.rs"), USERS_BIO).unwrap();
    run(&entities, &path, false, DatabaseEngine::SQLite).unwrap();
    assert_eq!(generated(&path).len(), 2);

    fs::write(format!("{entities}/user.rs"), USERS).unwrap();
    run(&entities, &path, false, DatabaseEngine::SQLite).unwrap();

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
    run(&entities, &path, false, DatabaseEngine::SQLite).unwrap();
    applied(&entities, &path);
    let create = generated(&path).remove(0);

    fs::write(format!("{entities}/user.rs"), USERS_BIO).unwrap();
    run(&entities, &path, false, DatabaseEngine::SQLite).unwrap();
    fs::write(format!("{entities}/user.rs"), USERS_NO_EMAIL).unwrap();
    run(&entities, &path, true, DatabaseEngine::SQLite).unwrap();

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
    run(&entities, &path, false, DatabaseEngine::SQLite).unwrap();
    applied(&entities, &path);
    fs::write(format!("{entities}/user.rs"), USERS_BIO).unwrap();
    run(&entities, &path, false, DatabaseEngine::SQLite).unwrap();
    let before = everything(&dir);

    fs::write(format!("{entities}/user.rs"), USERS_NO_EMAIL).unwrap();
    assert!(run(&entities, &path, false, DatabaseEngine::SQLite).is_err());

    fs::write(format!("{entities}/user.rs"), USERS_BIO).unwrap();
    assert_eq!(everything(&dir), before);
}

// A waiting migration edited by hand is computed again and overwritten.
#[test]
fn a_waiting_migration_edited_by_hand_is_overwritten() {
    let dir = TempDir::new();
    let (entities, path) = project(&dir);
    fs::write(format!("{entities}/user.rs"), USERS).unwrap();
    run(&entities, &path, false, DatabaseEngine::SQLite).unwrap();
    let (name, content) = generated(&path).remove(0);

    fs::write(format!("{path}/{name}"), format!("{content}// by hand\n")).unwrap();
    run(&entities, &path, false, DatabaseEngine::SQLite).unwrap();

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
    assert_eq!(
        started_from("// runique: from none\nfn up() {}"),
        Some("none")
    );
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
        table: crate::migration::TableChange::Altered,
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
            runique_i18n::keys::makemigrations::error::PARALLEL_CHANGES,
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
    run(&entities, &path, false, DatabaseEngine::SQLite).unwrap();
    let create = generated(&path).remove(0);
    assert_eq!(started_from(&create.1), Some("none"));
    applied(&entities, &path);

    fs::write(format!("{entities}/user.rs"), USERS_BIO).unwrap();
    run(&entities, &path, false, DatabaseEngine::SQLite).unwrap();
    run(&entities, &path, false, DatabaseEngine::SQLite).unwrap();
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
    run(&entities, &path, false, DatabaseEngine::SQLite).unwrap();
    applied(&entities, &path);
    let create = generated(&path)
        .remove(0)
        .0
        .trim_end_matches(".rs")
        .to_string();
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

    let error = run(&entities, &path, false, DatabaseEngine::SQLite)
        .unwrap_err()
        .to_string();

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
        names(dropped_tables(snapshots(), &[schema("menus", &[])], &[])),
        ["categories", "plats"]
    );
    assert_eq!(
        names(dropped_tables(snapshots(), &[], &[])),
        ["categories", "menus", "plats"]
    );
    assert!(dropped_tables(vec![], &[schema("menus", &[])], &[]).is_empty());
}

// One line per table, the wording the guard shows.
#[test]
fn a_dropped_table_is_announced_as_a_loss_of_data() {
    assert_eq!(
        dropped_table_messages(&["menus".to_string(), "plats".to_string()]),
        [
            "  menus: DROP TABLE (data loss)",
            "  plats: DROP TABLE (data loss)"
        ]
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
        order(vec![
            schema("a_parent", &[]),
            schema("z_child", &["a_parent"])
        ]),
        ["z_child", "a_parent"]
    );
    assert_eq!(
        order(vec![
            schema("z_parent", &[]),
            schema("a_child", &["z_parent"])
        ]),
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
    assert!(
        content.starts_with(&format!("{WAIT_MIGRATION}\n")),
        "{content}"
    );
    assert_eq!(
        started_from(content),
        Some("m20250101_000000_create_menus_table")
    );
    let (up, down) = content.split_once("async fn down").unwrap();
    assert!(up.contains(".drop_table(Table::drop()"), "{up}");
    assert!(up.contains(".if_exists()"), "{up}");
    assert!(!up.contains("create_table"), "{up}");
    assert!(down.contains("Table::create()"), "{down}");
    assert!(
        down.contains("menus_categories_id_categories_fkey"),
        "{down}"
    );
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
    run(&entities, &path, false, DatabaseEngine::SQLite).unwrap();
    applied(&entities, &path);

    fs::remove_file(format!("{entities}/post.rs")).unwrap();
    fs::write(format!("{entities}/user.rs"), USERS_BIO).unwrap();
    let before = everything(&dir);

    assert!(run(&entities, &path, false, DatabaseEngine::SQLite).is_err());
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
    run(&entities, &path, false, DatabaseEngine::SQLite).unwrap();
    applied(&entities, &path);
    fs::remove_file(format!("{entities}/post.rs")).unwrap();

    run(&entities, &path, true, DatabaseEngine::SQLite).unwrap();
    let files = generated(&path);
    let drop = files
        .iter()
        .find(|(name, _)| name.ends_with("_drop_posts_table.rs"))
        .expect("the drop migration");
    assert!(has_marker(&drop.1, WAIT_MIGRATION));
    assert!(Path::new(&snapshot_file_path(&path, "posts")).exists());

    // Case 13: the same drop is computed again, not a file moves.
    let waiting = everything(&dir);
    run(&entities, &path, true, DatabaseEngine::SQLite).unwrap();
    assert_eq!(everything(&dir), waiting);

    applied(&entities, &path);
    assert!(!Path::new(&snapshot_file_path(&path, "posts")).exists());
    assert!(Path::new(&snapshot_file_path(&path, "users")).exists());
    assert!(!crate::atomic::interrupted(Path::new(&path)));

    // Applied and gone: nothing more to do, with or without `--force`.
    let settled = everything(&dir);
    run(&entities, &path, false, DatabaseEngine::SQLite).unwrap();
    assert_eq!(everything(&dir), settled);
}

// The table a `down` brings back has its columns in the order the model
// declared them, not in the order the snapshot reader returns them.
#[test]
fn a_dropped_table_comes_back_with_its_columns_in_order() {
    let dir = TempDir::new();
    let (entities, path) = project(&dir);
    fs::write(format!("{entities}/user.rs"), USERS_BIO).unwrap();
    run(&entities, &path, false, DatabaseEngine::SQLite).unwrap();
    applied(&entities, &path);
    let created = generated(&path).remove(0).1;
    fs::remove_file(format!("{entities}/user.rs")).unwrap();

    run(&entities, &path, true, DatabaseEngine::SQLite).unwrap();

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
    run(&entities, &path, false, DatabaseEngine::SQLite).unwrap();
    applied(&entities, &path);
    let settled = everything(&dir);

    fs::remove_file(format!("{entities}/post.rs")).unwrap();
    run(&entities, &path, true, DatabaseEngine::SQLite).unwrap();
    assert_eq!(generated(&path).len(), 2);

    fs::write(format!("{entities}/post.rs"), POSTS).unwrap();
    run(&entities, &path, false, DatabaseEngine::SQLite).unwrap();
    assert_eq!(everything(&dir), settled);
}

// Case 10: the last model of the project is removed. Same treatment, and
// `migration up` then records it without a single model left to read.
#[test]
fn the_last_table_of_a_project_is_dropped_like_any_other() {
    let dir = TempDir::new();
    let (entities, path) = project(&dir);
    fs::write(format!("{entities}/post.rs"), POSTS).unwrap();
    run(&entities, &path, false, DatabaseEngine::SQLite).unwrap();
    applied(&entities, &path);
    fs::remove_file(format!("{entities}/post.rs")).unwrap();

    assert!(run(&entities, &path, false, DatabaseEngine::SQLite).is_err());
    run(&entities, &path, true, DatabaseEngine::SQLite).unwrap();
    fs::remove_dir(&entities).unwrap();
    crate::migration::steps::apply::migrate::record_applied(&entities, &path).unwrap();

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
    run(&entities, &path, false, DatabaseEngine::SQLite).unwrap();
    applied(&entities, &path);

    fs::remove_file(format!("{entities}/categorie.rs")).unwrap();
    fs::write(format!("{entities}/menu.rs"), MENUS_ALONE).unwrap();
    run(&entities, &path, true, DatabaseEngine::SQLite).unwrap();

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
    assert!(
        alter.contains("runtime::drop_foreign_key(manager, \"menus\""),
        "{alter}"
    );
}

// Case 26: the model that stays still points to the removed one. Making the
// models agree is the developer's job: the command stops, `--force` or not.
#[test]
fn a_model_still_pointing_to_a_removed_one_stops_the_command() {
    let dir = TempDir::new();
    let (entities, path) = project(&dir);
    fs::write(format!("{entities}/categorie.rs"), CATEGORIES).unwrap();
    fs::write(format!("{entities}/menu.rs"), MENUS_IN_CATEGORY).unwrap();
    run(&entities, &path, false, DatabaseEngine::SQLite).unwrap();
    applied(&entities, &path);
    fs::remove_file(format!("{entities}/categorie.rs")).unwrap();
    let before = everything(&dir);

    for force in [false, true] {
        let error = run(&entities, &path, force, DatabaseEngine::SQLite)
            .unwrap_err()
            .to_string();
        assert!(error.contains("categorie"), "{error}");
        assert_eq!(everything(&dir), before);
    }
}

// ── `managed: false` ─────────────────────────────────────────────────────────

const USERS_PAUSED: &str = r#"model! { User, table: "users", pk: id => i32, { username: text [required], email: text [required], }, meta: { managed: false } }"#;
const USERS_BIO_PAUSED: &str = r#"model! { User, table: "users", pk: id => i32, { username: text [required], email: text [required], bio: text [nullable], }, meta: { managed: false } }"#;

// A model is managed unless its `meta` says `managed: false`: no `meta`, a
// `meta` without the key and `managed: true` all leave it managed.
#[test]
fn a_model_is_managed_unless_its_meta_says_otherwise() {
    let managed = |source: &str| {
        crate::migration::parse_model_from_source(source)
            .unwrap()
            .unwrap()
            .managed
    };
    assert!(managed(USERS));
    assert!(managed(
        r#"model! { User, table: "users", pk: id => i32, { username: text [required] }, meta: { ordering: [username] } }"#
    ));
    assert!(managed(
        r#"model! { User, table: "users", pk: id => i32, { username: text [required] }, meta: { managed: true } }"#
    ));
    assert!(!managed(USERS_PAUSED));
}

// A paused table is the only one a snapshot without a schema keeps: it is
// not dropped, the other tables without a model still are.
#[test]
fn a_paused_table_is_not_to_drop() {
    let snapshots = || vec![schema("plats", &[]), schema("menus", &[])];
    let names = |dropped: Vec<ParsedSchema>| -> Vec<String> {
        dropped.into_iter().map(|s| s.table_name).collect()
    };

    assert_eq!(
        names(dropped_tables(snapshots(), &[], &["menus".to_string()])),
        ["plats"]
    );
    assert_eq!(
        names(dropped_tables(snapshots(), &[], &["other".to_string()])),
        ["menus", "plats"]
    );
}

// The scan sets a paused model aside by its table; the others are returned
// as before.
#[test]
fn the_scan_sets_a_paused_model_aside() {
    let dir = TempDir::new();
    let (entities, _) = project(&dir);
    fs::write(format!("{entities}/user.rs"), USERS_PAUSED).unwrap();
    fs::write(format!("{entities}/post.rs"), POSTS).unwrap();

    let ScannedEntities {
        schemas, paused, ..
    } = scan_models(&entities).unwrap();

    let tables: Vec<&str> = schemas.iter().map(|s| s.table_name.as_str()).collect();
    assert_eq!(tables, ["posts"]);
    assert_eq!(paused, ["users"]);
    let scanned = scan_entities(&entities).unwrap();
    assert_eq!(scanned.len(), 1);
    assert_eq!(scanned[0].table_name, "posts");
}

// Without a paused model the list is empty.
#[test]
fn the_scan_of_managed_models_pauses_nothing() {
    let dir = TempDir::new();
    let (entities, _) = project(&dir);
    fs::write(format!("{entities}/user.rs"), USERS).unwrap();

    let ScannedEntities {
        schemas, paused, ..
    } = scan_models(&entities).unwrap();

    assert_eq!(schemas.len(), 1);
    assert!(paused.is_empty());
}

// A key to a paused model still finds its real table and primary key.
#[test]
fn a_key_to_a_paused_model_is_still_resolved() {
    let dir = TempDir::new();
    let (entities, _) = project(&dir);
    fs::write(
        format!("{entities}/shelf.rs"),
        r#"model! { Shelf, table: "library_shelves", pk: code => i32, { label: text [required] }, meta: { managed: false } }"#,
    )
    .unwrap();
    fs::write(
        format!("{entities}/book.rs"),
        r#"model! { Book, table: "library_books", pk: id => i32, { shelf_code: int [required] },
            relations: { belongs_to: shelf via shelf_code [cascade] } }"#,
    )
    .unwrap();

    let ScannedEntities {
        schemas, paused, ..
    } = scan_models(&entities).unwrap();

    assert_eq!(paused, ["library_shelves"]);
    assert_eq!(schemas.len(), 1);
    let fk = &schemas[0].foreign_keys[0];
    assert_eq!(
        (fk.to_table.as_str(), fk.to_column.as_str()),
        ("library_shelves", "code")
    );
}

// The tables of a paused model's list fields are paused with it.
#[test]
fn the_list_tables_of_a_paused_model_are_paused_with_it() {
    let dir = TempDir::new();
    let (entities, _) = project(&dir);
    fs::write(
        format!("{entities}/book.rs"),
        r#"model! { Book, table: "books", pk: id => i32, enums: { Genre: [Roman, Policier] },
            { title: text [required], genres: checkbox [enum(Genre)] }, meta: { managed: false } }"#,
    )
    .unwrap();

    let ScannedEntities {
        schemas, paused, ..
    } = scan_models(&entities).unwrap();

    assert!(schemas.is_empty(), "{schemas:?}");
    assert_eq!(paused, ["books", "books_genres"]);
}

// One model, one table: a second model for the same table is refused, whether
// the paused one is read first, read last, or both are paused.
#[test]
fn two_models_for_one_table_are_refused_even_when_one_is_paused() {
    const OTHER: &str =
        r#"model! { Member, table: "users", pk: id => i32, { name: text [required] } }"#;
    const OTHER_PAUSED: &str = r#"model! { Member, table: "users", pk: id => i32, { name: text [required] }, meta: { managed: false } }"#;

    for (user, member) in [
        (USERS_PAUSED, OTHER),
        (USERS, OTHER_PAUSED),
        (USERS_PAUSED, OTHER_PAUSED),
    ] {
        let dir = TempDir::new();
        let (entities, path) = project(&dir);
        fs::write(format!("{entities}/user.rs"), user).unwrap();
        fs::write(format!("{entities}/member.rs"), member).unwrap();

        let err = scan_models(&entities).err().expect("two `users`");
        assert!(format!("{err:#}").contains("`users`"), "{err:#}");
        assert!(run(&entities, &path, true, DatabaseEngine::SQLite).is_err());
        assert!(generated(&path).is_empty());
    }
}

// A new model that is paused: no migration, with or without `--force`, and
// the managed model next to it is migrated as usual.
#[test]
fn nothing_is_generated_for_a_paused_model() {
    let dir = TempDir::new();
    let (entities, path) = project(&dir);
    fs::write(format!("{entities}/user.rs"), USERS_PAUSED).unwrap();

    run(&entities, &path, false, DatabaseEngine::SQLite).unwrap();
    assert!(generated(&path).is_empty());
    run(&entities, &path, true, DatabaseEngine::SQLite).unwrap();
    assert!(generated(&path).is_empty());

    fs::write(format!("{entities}/post.rs"), POSTS).unwrap();
    run(&entities, &path, false, DatabaseEngine::SQLite).unwrap();
    let files = generated(&path);
    assert_eq!(files.len(), 1, "{files:?}");
    assert!(files[0].0.ends_with("_create_posts_table.rs"), "{files:?}");
}

// A migrated table whose model is paused: not dropped, even with `--force`,
// and a change to the model generates nothing. Its snapshot stays as it was.
#[test]
fn a_migrated_table_put_on_pause_is_neither_dropped_nor_altered() {
    let dir = TempDir::new();
    let (entities, path) = project(&dir);
    fs::write(format!("{entities}/user.rs"), USERS).unwrap();
    run(&entities, &path, false, DatabaseEngine::SQLite).unwrap();
    applied(&entities, &path);
    // Everything but the models, which the test rewrites itself.
    let migration_side = || -> Vec<(String, String)> {
        everything(&dir)
            .into_iter()
            .filter(|(file, _)| !file.starts_with("entities"))
            .collect()
    };
    let before = migration_side();
    assert!(before.iter().any(|(file, _)| file.contains("snapshots")));

    fs::write(format!("{entities}/user.rs"), USERS_PAUSED).unwrap();
    run(&entities, &path, true, DatabaseEngine::SQLite).unwrap();
    assert_eq!(migration_side(), before);

    fs::write(format!("{entities}/user.rs"), USERS_BIO_PAUSED).unwrap();
    run(&entities, &path, true, DatabaseEngine::SQLite).unwrap();
    assert_eq!(migration_side(), before);
}

// The pause is lifted: the difference between the snapshot and the model
// comes out as an alteration, as for any table.
#[test]
fn a_table_taken_off_pause_is_migrated_from_its_snapshot() {
    let dir = TempDir::new();
    let (entities, path) = project(&dir);
    fs::write(format!("{entities}/user.rs"), USERS).unwrap();
    run(&entities, &path, false, DatabaseEngine::SQLite).unwrap();
    applied(&entities, &path);
    fs::write(format!("{entities}/user.rs"), USERS_BIO_PAUSED).unwrap();
    run(&entities, &path, false, DatabaseEngine::SQLite).unwrap();
    assert_eq!(generated(&path).len(), 1);

    fs::write(format!("{entities}/user.rs"), USERS_BIO).unwrap();
    run(&entities, &path, false, DatabaseEngine::SQLite).unwrap();

    let files = generated(&path);
    assert_eq!(files.len(), 2, "{files:?}");
    let alter = files
        .iter()
        .find(|(name, _)| name.ends_with("_alter_users_table.rs"))
        .expect("an alteration");
    assert!(alter.1.contains("bio"), "{}", alter.1);
}

// A migration still waiting for a table that is then paused goes at the next
// run: nothing is generated for a paused table.
#[test]
fn a_waiting_migration_of_a_table_put_on_pause_is_removed() {
    let dir = TempDir::new();
    let (entities, path) = project(&dir);
    fs::write(format!("{entities}/user.rs"), USERS).unwrap();
    run(&entities, &path, false, DatabaseEngine::SQLite).unwrap();
    assert_eq!(generated(&path).len(), 1);

    fs::write(format!("{entities}/user.rs"), USERS_PAUSED).unwrap();
    run(&entities, &path, false, DatabaseEngine::SQLite).unwrap();

    assert!(generated(&path).is_empty());
    let lib = fs::read_to_string(lib_path(&path)).unwrap();
    assert!(!lib.contains("users_table"), "{lib}");
}

// ── A table renamed with `renamed_from` ──────────────────────────────────────

const OWNER: &str =
    r#"model! { Owner, table: "owner", pk: id => i32, { name: text [required], } }"#;
const SHOP: &str = r#"model! { Shop, table: "shop", pk: id => i32, { name: text [required], city: text [required], owner_id: int [required] },
    relations: { belongs_to: owner via owner_id [cascade] }, meta: { indexes: [(city)] } }"#;
// The same model once its table is called `bakery`.
const BAKERY: &str = r#"model! { Shop, table: "bakery", renamed_from: "shop", pk: id => i32, { name: text [required], city: text [required], owner_id: int [required] },
    relations: { belongs_to: owner via owner_id [cascade] }, meta: { indexes: [(city)] } }"#;
// The new table name without saying where it comes from.
const BAKERY_UNDECLARED: &str = r#"model! { Shop, table: "bakery", pk: id => i32, { name: text [required], city: text [required], owner_id: int [required] },
    relations: { belongs_to: owner via owner_id [cascade] }, meta: { indexes: [(city)] } }"#;
const PRODUCT: &str = r#"model! { Product, table: "product", pk: id => i32, { label: text [required], shop_id: int [required] },
    relations: { belongs_to: shop via shop_id [cascade] } }"#;

/// A project whose `owner`, `shop` and `product` tables are applied.
fn shop_project(dir: &TempDir) -> (String, String) {
    let (entities, path) = project(dir);
    fs::write(format!("{entities}/owner.rs"), OWNER).unwrap();
    fs::write(format!("{entities}/shop.rs"), SHOP).unwrap();
    fs::write(format!("{entities}/product.rs"), PRODUCT).unwrap();
    run(&entities, &path, false, DatabaseEngine::SQLite).unwrap();
    applied(&entities, &path);
    (entities, path)
}

/// The generated migration of `path` whose file name ends with `suffix`.
fn generated_file(path: &str, suffix: &str) -> String {
    generated(path)
        .into_iter()
        .find(|(name, _)| name.ends_with(suffix))
        .unwrap_or_else(|| panic!("no `{suffix}` in {:?}", generated(path)))
        .1
}

fn up_and_down(content: &str) -> (&str, &str) {
    content.split_once("async fn down").expect("a `down`")
}

// The rename is one alteration of the table under its new name, without
// `--force`: the table is renamed, nothing is dropped, nothing is created.
#[test]
fn a_renamed_table_is_one_alteration_and_loses_nothing() {
    let dir = TempDir::new();
    let (entities, path) = shop_project(&dir);
    let before = generated(&path).len();

    fs::write(format!("{entities}/shop.rs"), BAKERY).unwrap();
    run(&entities, &path, false, DatabaseEngine::SQLite).unwrap();

    let files = generated(&path);
    // `bakery` itself, and `product`, whose key points to it.
    assert_eq!(files.len(), before + 2, "{files:?}");
    assert!(
        !files
            .iter()
            .any(|(name, _)| name.contains("_drop_") || name.ends_with("_create_bakery_table.rs")),
        "{files:?}"
    );
    let alter = generated_file(&path, "_alter_bakery_table.rs");
    assert!(has_marker(&alter, WAIT_MIGRATION));
    assert_eq!(renamed_table(&alter), Some("shop"));
    let (up, down) = up_and_down(&alter);
    assert!(
        up.contains(r#".table(Alias::new("shop"), Alias::new("bakery"))"#),
        "{up}"
    );
    assert!(
        down.contains(r#".table(Alias::new("bakery"), Alias::new("shop"))"#),
        "{down}"
    );
    assert!(!alter.contains("drop_table") && !alter.contains("create_table"));
    assert!(!alter.contains("drop_column") && !alter.contains("add_column"));
}

// The indexes and keys named after the former table take the new name, in
// `up`, and their former one back in `down`. Nothing asks the engine whether
// it can change keys: SQLite renames a table with its keys.
#[test]
fn the_indexes_and_keys_of_a_renamed_table_take_its_name() {
    let dir = TempDir::new();
    let (entities, path) = shop_project(&dir);

    fs::write(format!("{entities}/shop.rs"), BAKERY).unwrap();
    run(&entities, &path, false, DatabaseEngine::SQLite).unwrap();

    let alter = generated_file(&path, "_alter_bakery_table.rs");
    let (up, down) = up_and_down(&alter);
    // The index of the model, and the one of its key.
    for index in ["city", "owner_id"] {
        let (old, new) = (format!("idx_shop_{index}"), format!("idx_bakery_{index}"));
        let created = up
            .find(&format!(r#".name("{new}")"#))
            .expect("created in up");
        let dropped = up
            .find(&format!(r#".name("{old}")"#))
            .expect("dropped in up");
        assert!(created < dropped, "the new index before the old one goes");
        assert!(down.contains(&format!(r#".name("{old}")"#)), "{down}");
        assert!(down.contains(&format!(r#".name("{new}")"#)), "{down}");
    }
    // Every index statement runs on the table as it is called at that moment.
    assert!(!up.contains(r#".table(Alias::new("shop"))"#), "{up}");
    assert!(!down.contains(r#".table(Alias::new("shop"))"#), "{down}");

    let renamed_key = |body: &str, from: &str, to: &str| {
        body.contains(&format!(
            "rename_foreign_key(\n            manager,\n            \"bakery\",\n            \"{from}\",\n            ForeignKey::create()\n                .name(\"{to}\")"
        ))
    };
    assert!(
        renamed_key(up, "shop_owner_id_owner_fkey", "bakery_owner_id_owner_fkey"),
        "{up}"
    );
    assert!(
        renamed_key(
            down,
            "bakery_owner_id_owner_fkey",
            "shop_owner_id_owner_fkey"
        ),
        "{down}"
    );
    for refused_on_sqlite in [
        "key_changes_supported",
        "drop_foreign_key",
        "add_foreign_key",
    ] {
        assert!(!alter.contains(refused_on_sqlite), "{alter}");
    }

    // In `up` the table is renamed first, in `down` last.
    let rename_up = up.find("rename_table").unwrap();
    assert!(rename_up < up.find("create_index").unwrap());
    assert!(rename_up < up.find("rename_foreign_key").unwrap());
    let rename_down = down.find("rename_table").unwrap();
    assert!(rename_down > down.rfind("drop_index").unwrap());
    assert!(rename_down > down.rfind("rename_foreign_key").unwrap());
}

// A table that points to the renamed one: its key is renamed too, by its own
// migration, which runs after the rename and names the table as it is then.
#[test]
fn the_key_of_a_table_pointing_to_a_renamed_one_follows() {
    let dir = TempDir::new();
    let (entities, path) = shop_project(&dir);

    fs::write(format!("{entities}/shop.rs"), BAKERY).unwrap();
    run(&entities, &path, false, DatabaseEngine::SQLite).unwrap();

    let product = generated_file(&path, "_alter_product_table.rs");
    assert_eq!(renamed_table(&product), None);
    let (up, down) = up_and_down(&product);
    assert!(up.contains(r#""product_shop_id_shop_fkey","#), "{up}");
    assert!(
        up.contains(r#".name("product_shop_id_bakery_fkey")"#),
        "{up}"
    );
    assert!(
        up.contains(r#".to(Alias::new("bakery"), Alias::new("id"))"#),
        "{up}"
    );
    assert!(down.contains(r#""product_shop_id_bakery_fkey","#), "{down}");
    assert!(
        down.contains(r#".name("product_shop_id_shop_fkey")"#),
        "{down}"
    );
    // Undone before the table takes its former name back.
    assert!(
        down.contains(r#".to(Alias::new("bakery"), Alias::new("id"))"#),
        "{down}"
    );
    assert!(!product.contains("rename_table"), "{product}");
    assert!(!product.contains("key_changes_supported"), "{product}");
    // Its own index doesn't hold the renamed table's name: left alone.
    assert!(!product.contains("create_index") && !product.contains("drop_index"));

    let order = run_order(&path);
    let at = |kind: &str| order.iter().position(|m| m.ends_with(kind)).unwrap();
    assert!(
        at("_alter_bakery_table") < at("_alter_product_table"),
        "{order:?}"
    );
}

// Applied: the snapshot is under the new name, none is left under the former
// one, and the declaration left in the model does nothing any more.
#[test]
fn an_applied_rename_moves_the_snapshot_and_is_not_generated_again() {
    let dir = TempDir::new();
    let (entities, path) = shop_project(&dir);
    fs::write(format!("{entities}/shop.rs"), BAKERY).unwrap();
    run(&entities, &path, false, DatabaseEngine::SQLite).unwrap();

    // Run again before applying: the same rename, nothing rewritten.
    let waiting = everything(&dir);
    run(&entities, &path, false, DatabaseEngine::SQLite).unwrap();
    assert_eq!(everything(&dir), waiting);
    assert!(Path::new(&snapshot_file_path(&path, "shop")).exists());

    applied(&entities, &path);
    assert!(!Path::new(&snapshot_file_path(&path, "shop")).exists());
    assert!(Path::new(&snapshot_file_path(&path, "bakery")).exists());
    let product = fs::read_to_string(snapshot_file_path(&path, "product")).unwrap();
    assert!(product.contains(r#"Alias::new("bakery")"#), "{product}");
    assert!(!product.contains(r#"Alias::new("shop")"#), "{product}");

    let settled = everything(&dir);
    run(&entities, &path, true, DatabaseEngine::SQLite).unwrap();
    assert_eq!(everything(&dir), settled);
}

// Without the declaration the same change is a table dropped and another
// created: blocked without `--force`, and nothing is written.
#[test]
fn a_table_name_changed_without_saying_so_is_a_drop_and_a_creation() {
    let dir = TempDir::new();
    let (entities, path) = shop_project(&dir);
    let before = everything(&dir);

    fs::write(format!("{entities}/shop.rs"), BAKERY_UNDECLARED).unwrap();
    // The list (`shop: DROP TABLE`) goes to the console.
    assert!(run(&entities, &path, false, DatabaseEngine::SQLite).is_err());

    let after: Vec<_> = everything(&dir)
        .into_iter()
        .filter(|(file, _)| !file.ends_with("shop.rs"))
        .collect();
    let before: Vec<_> = before
        .into_iter()
        .filter(|(file, _)| !file.ends_with("shop.rs"))
        .collect();
    assert_eq!(after, before);
}

// A rename and a real change in the same run: both are in the one file, the
// change under the table's new name.
#[test]
fn a_rename_and_a_change_share_the_file_of_the_table() {
    let dir = TempDir::new();
    let (entities, path) = shop_project(&dir);

    fs::write(
        format!("{entities}/shop.rs"),
        BAKERY.replace(
            "city: text [required],",
            "city: text [required], phone: text [nullable],",
        ),
    )
    .unwrap();
    run(&entities, &path, false, DatabaseEngine::SQLite).unwrap();

    let alter = generated_file(&path, "_alter_bakery_table.rs");
    let (up, _) = up_and_down(&alter);
    assert!(up.find("rename_table").unwrap() < up.find("add_column").unwrap());
    assert!(up.contains(r#"Alias::new("phone")"#), "{up}");
    assert!(
        !generated(&path)
            .iter()
            .any(|(name, _)| name.ends_with("_alter_shop_table.rs"))
    );
}

const BOOK_GENRES: &str = r#"model! { Book, table: "books", pk: id => i32, enums: { Genre: [Roman, Policier] },
    { title: text [required], genres: checkbox [enum(Genre)] } }"#;

/// A project whose `books` and `books_genres` tables are applied, then the
/// model replaced by `model` and the command run again.
fn renamed_list(dir: &TempDir, model: &str) -> String {
    let (entities, path) = project(dir);
    fs::write(format!("{entities}/book.rs"), BOOK_GENRES).unwrap();
    run(&entities, &path, false, DatabaseEngine::SQLite).unwrap();
    applied(&entities, &path);
    fs::write(format!("{entities}/book.rs"), model).unwrap();
    run(&entities, &path, false, DatabaseEngine::SQLite).unwrap();
    path
}

// The table of a list field is named after its model's table and after the
// field: it follows the rename of either, and of both at once.
#[test]
fn a_list_table_follows_its_model_and_its_field() {
    for (model, table) in [
        (
            BOOK_GENRES.replace(
                r#"table: "books""#,
                r#"table: "library", renamed_from: "books""#,
            ),
            "library_genres",
        ),
        (
            BOOK_GENRES.replace(
                "genres: checkbox [enum(Genre)]",
                r#"styles: checkbox [enum(Genre), renamed_from: "genres"]"#,
            ),
            "books_styles",
        ),
        (
            BOOK_GENRES
                .replace(
                    r#"table: "books""#,
                    r#"table: "library", renamed_from: "books""#,
                )
                .replace(
                    "genres: checkbox [enum(Genre)]",
                    r#"styles: checkbox [enum(Genre), renamed_from: "genres"]"#,
                ),
            "library_styles",
        ),
    ] {
        let dir = TempDir::new();
        let path = renamed_list(&dir, &model);

        let list = generated_file(&path, &format!("_alter_{table}_table.rs"));
        assert_eq!(renamed_table(&list), Some("books_genres"), "{table}");
        let (up, _) = up_and_down(&list);
        assert!(
            up.contains(&format!(
                r#".table(Alias::new("books_genres"), Alias::new("{table}"))"#
            )),
            "{up}"
        );
        assert!(
            up.contains(&format!(r#".name("{table}_owner_id_value_uniq")"#)),
            "{up}"
        );
        assert!(
            up.contains(r#".name("books_genres_owner_id_value_uniq")"#),
            "{up}"
        );
        assert!(!list.contains("drop_table") && !list.contains("create_table"));
        let files = generated(&path);
        assert!(
            !files.iter().any(|(name, _)| name.contains("_drop_")),
            "{files:?}"
        );
    }
}

// A list field renamed without saying so: its table is dropped and another
// created, blocked without `--force`.
#[test]
fn a_list_field_renamed_without_saying_so_is_a_drop() {
    let dir = TempDir::new();
    let (entities, path) = project(&dir);
    fs::write(format!("{entities}/book.rs"), BOOK_GENRES).unwrap();
    run(&entities, &path, false, DatabaseEngine::SQLite).unwrap();
    applied(&entities, &path);

    fs::write(
        format!("{entities}/book.rs"),
        BOOK_GENRES.replace("genres: checkbox", "styles: checkbox"),
    )
    .unwrap();
    let before = generated(&path);

    // The list (`books_genres: DROP TABLE`) goes to the console.
    assert!(run(&entities, &path, false, DatabaseEngine::SQLite).is_err());
    assert_eq!(generated(&path), before);

    // Forced: the former table is dropped, the new one created.
    run(&entities, &path, true, DatabaseEngine::SQLite).unwrap();
    let files = generated(&path);
    assert!(
        files
            .iter()
            .any(|(name, _)| name.ends_with("_drop_books_genres_table.rs")),
        "{files:?}"
    );
    assert!(
        files
            .iter()
            .any(|(name, _)| name.ends_with("_create_books_styles_table.rs")),
        "{files:?}"
    );
}

// A declared rename holds only while the former name has a snapshot and no
// model, and the new name has no snapshot.
#[test]
fn a_declared_rename_holds_against_the_snapshots() {
    let snapshot = |table: &str| generate_snapshot_file(&schema(table, &[]));
    let declared = [("shop".to_string(), "bakery".to_string())];
    let holds = |files: &[(&str, &str)], schemas: &[ParsedSchema], paused: &[String]| {
        let dir = TempDir::new();
        let path = with_snapshots(&dir, files);
        effective_renames(&declared, schemas, paused, &path).unwrap()
    };
    let (shop, bakery) = (snapshot("shop"), snapshot("bakery"));
    let (shop, bakery) = (shop.as_str(), bakery.as_str());

    assert_eq!(
        holds(&[("shop.rs", shop)], &[schema("bakery", &[])], &[]),
        declared
    );
    // Applied already: the snapshot is under the new name.
    assert!(holds(&[("bakery.rs", bakery)], &[schema("bakery", &[])], &[]).is_empty());
    assert!(
        holds(
            &[("shop.rs", shop), ("bakery.rs", bakery)],
            &[schema("bakery", &[])],
            &[]
        )
        .is_empty()
    );
    // Never migrated under the former name.
    assert!(holds(&[], &[schema("bakery", &[])], &[]).is_empty());
    // The former name still has a model, managed or paused.
    assert!(
        holds(
            &[("shop.rs", shop)],
            &[schema("bakery", &[]), schema("shop", &[])],
            &[]
        )
        .is_empty()
    );
    assert!(
        holds(
            &[("shop.rs", shop)],
            &[schema("bakery", &[])],
            &["shop".to_string()]
        )
        .is_empty()
    );
}

// Two tables can't both come from one: the error names the three.
#[test]
fn two_tables_renamed_from_one_are_refused() {
    let dir = TempDir::new();
    let shop = generate_snapshot_file(&schema("shop", &[]));
    let path = with_snapshots(&dir, &[("shop.rs", shop.as_str())]);
    let declared = [
        ("shop".to_string(), "bakery".to_string()),
        ("shop".to_string(), "pastry".to_string()),
    ];

    let error = effective_renames(
        &declared,
        &[schema("bakery", &[]), schema("pastry", &[])],
        &[],
        &path,
    )
    .unwrap_err()
    .to_string();

    for table in ["`shop`", "`bakery`", "`pastry`"] {
        assert!(error.contains(table), "{error}");
    }
}

fn named_index(name: &str, columns: &[&str], unique: bool) -> crate::migration::types::ParsedIndex {
    crate::migration::types::ParsedIndex {
        name: name.to_string(),
        columns: columns.iter().map(|c| (*c).to_string()).collect(),
        unique,
    }
}

// What a rename explains leaves the diff: the snapshot of the former name,
// brought in line, shows no change.
#[test]
fn a_rename_alone_leaves_the_diff_empty() {
    let mut previous = ParsedSchema {
        indexes: vec![
            named_index("idx_shop_title", &["title"], false),
            named_index("shop_title_uniq", &["title"], true),
        ],
        ..schema("shop", &["owner", "shop"])
    };
    let current = ParsedSchema {
        indexes: vec![
            named_index("idx_bakery_title", &["title"], false),
            named_index("bakery_title_uniq", &["title"], true),
        ],
        ..schema("bakery", &["owner"])
    };
    // The key a table has to itself points to its new name.
    let current = ParsedSchema {
        foreign_keys: vec![
            current.foreign_keys[0].clone(),
            ParsedFk {
                from_column: "shop_id".into(),
                to_table: "bakery".into(),
                ..current.foreign_keys[0].clone()
            },
        ],
        ..current
    };
    let renames = [("shop".to_string(), "bakery".to_string())];

    let followed = follow_renames(&mut previous, &current, &renames);

    assert_eq!(followed.from.as_deref(), Some("shop"));
    let keys: Vec<(&str, &str, &str, &str)> = followed
        .keys
        .iter()
        .map(|k| {
            (
                k.old_table.as_str(),
                k.old_target.as_str(),
                k.key.to_table.as_str(),
                k.target_at_run.as_str(),
            )
        })
        .collect();
    assert_eq!(
        keys,
        [
            ("shop", "owner", "owner", "owner"),
            ("shop", "shop", "bakery", "bakery")
        ]
    );
    let indexes: Vec<(&str, &str)> = followed
        .indexes
        .iter()
        .map(|(old, new)| (old.name.as_str(), new.name.as_str()))
        .collect();
    assert_eq!(
        indexes,
        [
            ("idx_shop_title", "idx_bakery_title"),
            ("shop_title_uniq", "bakery_title_uniq")
        ]
    );
    assert_eq!(previous.table_name, "bakery");
    assert!(diff_schemas(&previous, &current).is_empty());
    // Not empty as a whole: the rename is a change.
    let changes = Changes {
        renames: followed,
        ..diff_schemas(&previous, &current)
    };
    assert!(!changes.is_empty());
}

// A table that isn't renamed: only its keys to a renamed table follow, its
// indexes and its other keys are left to the diff.
#[test]
fn a_table_that_keeps_its_name_only_renames_its_keys_to_a_renamed_one() {
    let mut previous = ParsedSchema {
        indexes: vec![named_index("idx_product_old", &["title"], false)],
        ..schema("product", &["shop", "owner"])
    };
    let current = ParsedSchema {
        indexes: vec![named_index("idx_product_new", &["title"], false)],
        foreign_keys: vec![
            ParsedFk {
                from_column: "shop_id".into(),
                to_table: "bakery".into(),
                to_column: "id".into(),
                on_delete: crate::migration::FkAction::Cascade,
                on_update: crate::migration::FkAction::NoAction,
            },
            schema("product", &["owner"]).foreign_keys[0].clone(),
        ],
        ..schema("product", &[])
    };
    let renames = [("shop".to_string(), "bakery".to_string())];

    let followed = follow_renames(&mut previous, &current, &renames);

    assert_eq!(followed.from, None);
    assert_eq!(followed.keys.len(), 1);
    assert_eq!(
        (
            followed.keys[0].old_table.as_str(),
            followed.keys[0].old_target.as_str(),
            followed.keys[0].key.from_column.as_str()
        ),
        ("product", "shop", "shop_id")
    );
    assert!(followed.indexes.is_empty());
    let diff = diff_schemas(&previous, &current);
    assert!(diff.added_fks.is_empty() && diff.dropped_fks.is_empty());
    assert_eq!(diff.dropped_indexes.len(), 1);
    assert_eq!(diff.added_indexes.len(), 1);

    // Without any rename nothing follows.
    let mut untouched = schema("product", &["owner"]);
    let same = schema("product", &["owner"]);
    assert!(follow_renames(&mut untouched, &same, &[]).is_empty());
    assert!(follow_renames(&mut untouched, &same, &renames).is_empty());
}

// A key that changed otherwise is not a renamed key: the diff removes it
// under its old name and adds it under the new one. An index whose columns
// or uniqueness changed is not a renamed index either.
#[test]
fn a_key_or_an_index_that_really_changed_is_left_to_the_diff() {
    let mut previous = ParsedSchema {
        indexes: vec![
            named_index("idx_shop_title", &["title"], false),
            named_index("idx_shop_owner_id", &["owner_id"], false),
        ],
        ..schema("shop", &["owner"])
    };
    let mut current = ParsedSchema {
        indexes: vec![
            named_index("idx_bakery_title", &["title"], true),
            named_index("idx_bakery_owner_id", &["owner_id", "title"], false),
        ],
        ..schema("bakery", &["owner"])
    };
    current.foreign_keys[0].on_delete = crate::migration::FkAction::SetNull;
    let renames = [("shop".to_string(), "bakery".to_string())];

    let followed = follow_renames(&mut previous, &current, &renames);

    assert_eq!(followed.from.as_deref(), Some("shop"));
    assert!(followed.keys.is_empty());
    assert!(followed.indexes.is_empty());
    let diff = diff_schemas(&previous, &current);
    assert_eq!((diff.dropped_fks.len(), diff.added_fks.len()), (1, 1));
    assert_eq!(
        (diff.dropped_indexes.len(), diff.added_indexes.len()),
        (2, 2)
    );

    // In the file, the key that goes still has the name of the former table.
    let alter = generate_alter_file(&Changes {
        renames: followed,
        ..diff
    });
    let (up, down) = up_and_down(&alter);
    assert!(
        up.contains(r#""bakery", "shop_owner_id_owner_fkey")"#),
        "{up}"
    );
    assert!(
        up.contains(r#".name("bakery_owner_id_owner_fkey")"#),
        "{up}"
    );
    assert!(
        down.contains(r#".name("shop_owner_id_owner_fkey")"#),
        "{down}"
    );
    assert!(
        down.contains(r#""bakery", "bakery_owner_id_owner_fkey")"#),
        "{down}"
    );
    // Keys are added and removed: the engine is asked first.
    assert!(alter.contains("key_changes_supported"), "{alter}");
}

/// A change to the existing table `table`, renamed from `from` when given,
/// whose keys to `targets` are renamed.
fn renamed_change(table: &str, from: Option<&str>, targets: &[&str]) -> Changes {
    Changes {
        table: crate::migration::TableChange::Altered,
        added_columns: vec![],
        added_fks: vec![],
        renames: crate::migration::types::TableRenames {
            from: from.map(str::to_string),
            keys: schema(table, targets)
                .foreign_keys
                .into_iter()
                .map(|key| crate::migration::types::RenamedKey {
                    old_table: from.unwrap_or(table).to_string(),
                    old_target: key.to_table.clone(),
                    target_at_run: key.to_table.clone(),
                    key,
                })
                .collect(),
            indexes: vec![],
        },
        ..new_table(&schema(table, &[]))
    }
}

// A renamed key names its target as it is called when the migration runs:
// by its former name while the target isn't renamed yet.
#[test]
fn a_renamed_key_names_its_target_as_it_is_at_that_moment() {
    let targets = |changes: &[Changes]| -> Vec<Vec<String>> {
        changes
            .iter()
            .map(|c| {
                c.renames
                    .keys
                    .iter()
                    .map(|k| k.target_at_run.clone())
                    .collect()
            })
            .collect()
    };
    // `a` runs first: `b` still has its former name, `a` already its new one.
    let mut changes = vec![
        renamed_change("a", Some("a_old"), &["b", "a", "c"]),
        renamed_change("b", Some("b_old"), &["a"]),
        renamed_change("z", None, &["a", "b"]),
    ];

    settle_renamed_keys(&mut changes).unwrap();

    assert_eq!(
        targets(&changes),
        [vec!["b_old", "a", "c"], vec!["a"], vec!["a", "b"]]
    );
}

// A key that is new to a renamed table would be named for a target that
// isn't there yet: refused when the target is created or renamed later in
// the run, accepted otherwise.
#[test]
fn a_new_key_of_a_renamed_table_can_not_wait_for_its_target() {
    let with_new_key = |table: &str, from: Option<&str>, to: &str| Changes {
        added_fks: schema(table, &[to]).foreign_keys,
        ..renamed_change(table, from, &[])
    };
    let refused =
        |mut changes: Vec<Changes>| settle_renamed_keys(&mut changes).map_err(|e| e.to_string());

    // To a table this run creates.
    let error = refused(vec![
        with_new_key("a", Some("a_old"), "n"),
        new_table(&schema("n", &[])),
    ])
    .unwrap_err();
    assert!(error.contains("`a`") && error.contains("`n`"), "{error}");
    // To a table renamed later in the run.
    assert!(
        refused(vec![
            with_new_key("a", Some("a_old"), "b"),
            renamed_change("b", Some("b_old"), &[]),
        ])
        .is_err()
    );
    // To a table renamed before, to itself, or to a table that doesn't move.
    assert!(
        refused(vec![
            renamed_change("a", Some("a_old"), &[]),
            with_new_key("b", Some("b_old"), "a"),
        ])
        .is_ok()
    );
    assert!(refused(vec![with_new_key("a", Some("a_old"), "a")]).is_ok());
    assert!(refused(vec![with_new_key("a", Some("a_old"), "c")]).is_ok());
    // A table that keeps its name takes any new key: it runs after them all.
    assert!(
        refused(vec![
            renamed_change("b", Some("b_old"), &[]),
            new_table(&schema("n", &[])),
            with_new_key("z", None, "n"),
            with_new_key("y", None, "b"),
        ])
        .is_ok()
    );
}

// Renamed tables run before the tables created and the other alterations.
#[test]
fn renamed_tables_run_first() {
    let sorted = topological_sort_changes(vec![
        renamed_change("m", None, &[]),
        new_table(&schema("a", &[])),
        renamed_change("z", Some("z_old"), &[]),
        renamed_change("r", Some("r_old"), &[]),
    ]);
    let order: Vec<&str> = sorted.iter().map(|c| c.table_name.as_str()).collect();
    assert_eq!(order, ["r", "z", "a", "m"]);
}

// The marker of a renamed table: read wherever it is, and only when what
// follows is a plain table name, which becomes a file path.
#[test]
fn the_former_name_of_a_renamed_table_is_read_from_its_migration() {
    assert_eq!(
        renamed_table("// runique: waiting migrate up\n// runique: renamed table shop\nuse x;"),
        Some("shop")
    );
    assert_eq!(
        renamed_table("  // runique: renamed table shop_2  \n"),
        Some("shop_2")
    );
    assert_eq!(renamed_table("use x;\n// runique: from none\n"), None);
    for bad in ["../shop", "shop/x", "shop.rs", "", "a b"] {
        assert_eq!(
            renamed_table(&format!("{RENAMED_TABLE}{bad}\n")),
            None,
            "{bad:?}"
        );
    }
}

// The names a rename gives are checked like the others.
#[test]
fn a_rename_giving_a_name_too_long_is_reported() {
    let long = "t".repeat(40);
    let short = renamed_change("bakery", Some("shop"), &["owner"]);
    assert!(collect_long_identifier_messages(std::slice::from_ref(&short)).is_empty());

    let key_too_long = renamed_change(&long, Some("shop"), &[long.as_str()]);
    assert_eq!(collect_long_identifier_messages(&[key_too_long]).len(), 1);

    let mut index_too_long = renamed_change("bakery", Some("shop"), &[]);
    index_too_long.renames.indexes = vec![(
        named_index("idx_shop_title", &["title"], false),
        named_index(&"i".repeat(64), &["title"], false),
    )];
    assert_eq!(collect_long_identifier_messages(&[index_too_long]).len(), 1);
}

// 63 characters is the last length accepted for the name of a renamed key.
#[test]
fn a_renamed_key_name_is_accepted_up_to_63_characters() {
    let key = &renamed_change("t", Some("shop"), &["owner"]).renames.keys[0].key;
    // `{table}_{from}_{to}_fkey`
    let around = key.from_column.len() + key.to_table.len() + "__".len() + "_fkey".len();

    let at_limit = renamed_change(&"t".repeat(63 - around), Some("shop"), &["owner"]);
    assert!(collect_long_identifier_messages(&[at_limit]).is_empty());

    let one_over = renamed_change(&"t".repeat(64 - around), Some("shop"), &["owner"]);
    assert_eq!(collect_long_identifier_messages(&[one_over]).len(), 1);
}

// ── A primary key that changed since its snapshot ────────────────────────────

fn keyed(table: &str, key: Option<(&str, &str)>) -> ParsedSchema {
    ParsedSchema {
        primary_key: key.map(|(name, col_type)| ParsedColumn {
            name: name.into(),
            col_type: crate::migration::ColumnType::from_name(col_type)
                .expect("the name of a column type"),
            ..ParsedColumn::default()
        }),
        ..schema(table, &[])
    }
}

// Name, type, or both: one line naming the table and both keys. The same
// key, or a side without one, says nothing.
#[test]
fn a_primary_key_is_compared_by_its_name_and_its_type() {
    let was = keyed("users", Some(("id", "Integer")));
    let change = |now: Option<(&str, &str)>| primary_key_change(&was, &keyed("users", now));

    assert_eq!(change(Some(("id", "Integer"))), None);
    assert_eq!(
        change(Some(("id", "Uuid"))).as_deref(),
        Some("  users: id (Integer) -> id (Uuid)")
    );
    assert_eq!(
        change(Some(("uid", "Integer"))).as_deref(),
        Some("  users: id (Integer) -> uid (Integer)")
    );
    assert_eq!(
        change(Some(("uid", "BigInteger"))).as_deref(),
        Some("  users: id (Integer) -> uid (BigInteger)")
    );
    assert_eq!(change(None), None, "a model without a key");
    assert_eq!(
        primary_key_change(&keyed("users", None), &was),
        None,
        "a snapshot without a key"
    );
}

// The line names the table as the model calls it now.
#[test]
fn a_changed_key_is_reported_under_the_name_of_the_model() {
    let was = keyed("shop", Some(("id", "Integer")));
    let now = keyed("bakery", Some(("id", "Uuid")));
    assert_eq!(
        primary_key_change(&was, &now).as_deref(),
        Some("  bakery: id (Integer) -> id (Uuid)")
    );
}

/// `users` applied with an `i32` key, then its model rewritten by `change`.
fn users_with_another_key(dir: &TempDir, change: (&str, &str)) -> (String, String) {
    let (entities, path) = project(dir);
    fs::write(format!("{entities}/user.rs"), USERS).unwrap();
    run(&entities, &path, false, DatabaseEngine::SQLite).unwrap();
    applied(&entities, &path);
    let model = USERS.replace(change.0, change.1);
    assert_ne!(model, USERS);
    fs::write(format!("{entities}/user.rs"), model).unwrap();
    (entities, path)
}

// The whole command: a key whose type changed is refused, `--force` or not,
// and nothing is written.
#[test]
fn a_key_that_changed_type_is_refused_with_or_without_force() {
    let dir = TempDir::new();
    let (entities, path) = users_with_another_key(&dir, ("id => i32", "id => uuid"));
    let before = everything(&dir);

    for force in [false, true] {
        let error = run(&entities, &path, force, DatabaseEngine::SQLite)
            .unwrap_err()
            .to_string();
        assert!(
            error.contains("  users: id (Integer) -> id (Uuid)"),
            "{error}"
        );
        assert_eq!(everything(&dir), before, "force: {force}");
    }
}

#[test]
fn a_key_that_changed_name_is_refused() {
    let dir = TempDir::new();
    let (entities, path) = users_with_another_key(&dir, ("id => i32", "uid => i32"));
    let before = everything(&dir);

    let error = run(&entities, &path, true, DatabaseEngine::SQLite)
        .unwrap_err()
        .to_string();
    assert!(
        error.contains("  users: id (Integer) -> uid (Integer)"),
        "{error}"
    );
    assert_eq!(everything(&dir), before);
}

// The key put back as it was: the command runs again, and the other change
// of the model is migrated.
#[test]
fn the_former_key_put_back_lets_the_command_run() {
    let dir = TempDir::new();
    let (entities, path) = users_with_another_key(&dir, ("id => i32", "id => i64"));
    assert!(run(&entities, &path, false, DatabaseEngine::SQLite).is_err());

    fs::write(format!("{entities}/user.rs"), USERS_BIO).unwrap();
    run(&entities, &path, false, DatabaseEngine::SQLite).unwrap();
    assert_eq!(generated(&path).len(), 2);
}

// Every table is named in one refusal, not one per run; a table whose key
// didn't move is not.
#[test]
fn every_table_with_a_changed_key_is_named_at_once() {
    let dir = TempDir::new();
    let (entities, path) = project(&dir);
    fs::write(format!("{entities}/user.rs"), USERS).unwrap();
    fs::write(format!("{entities}/post.rs"), POSTS).unwrap();
    fs::write(format!("{entities}/owner.rs"), OWNER).unwrap();
    run(&entities, &path, false, DatabaseEngine::SQLite).unwrap();
    applied(&entities, &path);
    fs::write(
        format!("{entities}/user.rs"),
        USERS.replace("id => i32", "id => i64"),
    )
    .unwrap();
    fs::write(
        format!("{entities}/post.rs"),
        POSTS.replace("id => i32", "id => uuid"),
    )
    .unwrap();
    let before = everything(&dir);

    let error = run(&entities, &path, true, DatabaseEngine::SQLite)
        .unwrap_err()
        .to_string();

    assert!(
        error.contains("  users: id (Integer) -> id (BigInteger)"),
        "{error}"
    );
    assert!(
        error.contains("  posts: id (Integer) -> id (Uuid)"),
        "{error}"
    );
    assert!(!error.contains("owner"), "{error}");
    assert_eq!(everything(&dir), before);
}

// The refusal comes before the destructive guard: a column dropped in the
// same run is not what the dev is told about.
#[test]
fn a_changed_key_is_refused_before_the_destructive_changes_are_listed() {
    let dir = TempDir::new();
    let (entities, path) = project(&dir);
    fs::write(format!("{entities}/user.rs"), USERS_BIO).unwrap();
    run(&entities, &path, false, DatabaseEngine::SQLite).unwrap();
    applied(&entities, &path);
    let model = USERS_NO_EMAIL.replace("id => i32", "id => i64");
    fs::write(format!("{entities}/user.rs"), model).unwrap();

    let error = run(&entities, &path, false, DatabaseEngine::SQLite)
        .unwrap_err()
        .to_string();

    assert!(
        error.contains("  users: id (Integer) -> id (BigInteger)"),
        "{error}"
    );
    assert!(!error.contains("DROP COLUMN"), "{error}");
}

// A table renamed in the same run is compared to the snapshot of its former
// name, and named as the model calls it now.
#[test]
fn a_renamed_table_keeps_the_key_of_its_former_snapshot() {
    let dir = TempDir::new();
    let (entities, path) = shop_project(&dir);
    fs::write(
        format!("{entities}/shop.rs"),
        BAKERY.replace("id => i32", "id => i64"),
    )
    .unwrap();
    let before = everything(&dir);

    let error = run(&entities, &path, false, DatabaseEngine::SQLite)
        .unwrap_err()
        .to_string();

    assert!(
        error.contains("  bakery: id (Integer) -> id (BigInteger)"),
        "{error}"
    );
    assert_eq!(everything(&dir), before);
}

// `Pk` follows the project's feature: the day the project changes it, every
// key written `Pk` changes type, and the command says so instead of running.
#[test]
fn a_project_changing_its_key_feature_is_refused() {
    use runique_dsl::ast::PkType;
    use runique_dsl::with_project_pk;

    let dir = TempDir::new();
    let (entities, path) = project(&dir);
    let model = USERS.replace("id => i32", "id => Pk");
    fs::write(format!("{entities}/user.rs"), model).unwrap();
    with_project_pk(PkType::I32, || {
        run(&entities, &path, false, DatabaseEngine::SQLite)
    })
    .unwrap();
    // The snapshot is written from the model: read with the same feature.
    with_project_pk(PkType::I32, || applied(&entities, &path));
    let settled = everything(&dir);

    // The same feature: nothing changed.
    with_project_pk(PkType::I32, || {
        run(&entities, &path, false, DatabaseEngine::SQLite)
    })
    .unwrap();
    assert_eq!(everything(&dir), settled);

    for (ty, line) in [
        (PkType::I64, "  users: id (Integer) -> id (BigInteger)"),
        (PkType::Uuid, "  users: id (Integer) -> id (Uuid)"),
    ] {
        let error = with_project_pk(ty, || run(&entities, &path, true, DatabaseEngine::SQLite))
            .unwrap_err()
            .to_string();
        assert!(error.contains(line), "{error}");
        assert_eq!(everything(&dir), settled);
    }
}

// ── An `extend!{}` block removed since its columns were applied ──────────────

const PROFILE: &str = r#"extend! { table: "eihwaz_users", fields: { bio: text, website: url } }"#;
const PROFILE_BIO: &str = r#"extend! { table: "eihwaz_users", fields: { bio: text } }"#;
const PROFILE_EMPTY: &str = r#"extend! { table: "eihwaz_users", fields: { } }"#;
const TEAM: &str = r#"extend! { table: "eihwaz_groupes", fields: { motto: text } }"#;

/// A project whose `blocks` (file name, content) are applied: their
/// snapshots are there, as `migration up` writes them, and nothing waits.
/// Written directly: a second migration of the same table in the same second
/// would take the name of the first.
fn extended_project(dir: &TempDir, blocks: &[(&str, &str)]) -> (String, String) {
    let (entities, path) = project(dir);
    for (name, content) in blocks {
        fs::write(format!("{entities}/{name}"), content).unwrap();
    }
    fs::create_dir_all(extend_snapshot_dir(&path)).unwrap();
    for schema in merge_extend_schemas(scan_extend_blocks(&entities).unwrap()) {
        fs::write(
            extend_snapshot_file_path(&path, &schema.table_name),
            extend_snapshot_content(&schema),
        )
        .unwrap();
    }
    run(&entities, &path, false, DatabaseEngine::SQLite).unwrap();
    assert!(generated(&path).is_empty(), "nothing to migrate");
    (entities, path)
}

/// The names of the columns a run would drop, per table.
fn dropped_by_extensions(entities: &str, path: &str) -> Vec<(String, Vec<String>)> {
    plan_extend_changes(entities, path)
        .unwrap()
        .into_iter()
        .map(|(schema, changes)| {
            let mut columns: Vec<String> = changes
                .dropped_columns
                .iter()
                .map(|c| c.name.clone())
                .collect();
            columns.sort();
            assert!(changes.added_columns.is_empty(), "{}", schema.table_name);
            (schema.table_name, columns)
        })
        .collect()
}

// The tables of `snapshots/runique/`: the `.rs` files, by their name, sorted.
// No folder, no table.
#[test]
fn the_extended_tables_are_the_snapshots_of_their_folder() {
    let dir = TempDir::new();
    let path = migrations_src(&dir, None);
    assert!(extended_tables(&path).is_empty(), "no folder yet");

    let folder = extend_snapshot_dir(&path);
    fs::create_dir_all(format!("{folder}/a_folder")).unwrap();
    for name in ["eihwaz_users.rs", "eihwaz_groupes.rs", "notes.txt", "bare"] {
        fs::write(format!("{folder}/{name}"), "").unwrap();
    }
    // A snapshot of a model, one folder up, is not an extension.
    fs::write(snapshot_file_path(&path, "posts"), "").unwrap();

    assert_eq!(extended_tables(&path), ["eihwaz_groupes", "eihwaz_users"]);
}

// ── A folder named like a `.rs` file, where Runique reads files ──────────────

// Among the entities: the command stops by the folder's name, with or
// without `--force`, and writes nothing, the models around it included.
#[test]
fn a_folder_named_like_a_model_file_stops_the_command() {
    let dir = TempDir::new();
    let (entities, path) = project(&dir);
    fs::write(format!("{entities}/user.rs"), USERS).unwrap();
    fs::create_dir(format!("{entities}/archive.rs")).unwrap();
    let before = everything(&dir);

    for force in [false, true] {
        let error = run(&entities, &path, force, DatabaseEngine::SQLite)
            .unwrap_err()
            .to_string();
        assert!(error.contains("archive.rs"), "{error}");
        assert_eq!(everything(&dir), before, "force: {force}");
    }

    // Renamed: the model beside it is migrated.
    fs::rename(
        format!("{entities}/archive.rs"),
        format!("{entities}/archive"),
    )
    .unwrap();
    run(&entities, &path, false, DatabaseEngine::SQLite).unwrap();
    assert_eq!(generated(&path).len(), 1);
}

// Among the snapshots, of the models or of the extensions: refused the same.
#[test]
fn a_folder_named_like_a_snapshot_stops_the_reading() {
    for extension in [false, true] {
        let dir = TempDir::new();
        let path = migrations_src(&dir, None);
        let folder = if extension {
            extend_snapshot_dir(&path)
        } else {
            snapshot_dir(&path)
        };
        fs::create_dir_all(format!("{folder}/gone.rs")).unwrap();

        let error = read_all(&path).unwrap_err().to_string();

        assert!(error.contains("gone.rs"), "{error}");
    }
}

// No snapshot folder yet is a new project, not an error.
#[test]
fn a_project_without_snapshots_reads_none() {
    let dir = TempDir::new();
    let path = migrations_src(&dir, None);
    assert!(read_all(&path).unwrap().is_empty());
}

// Blocks on one table, in several files: their columns come in the order of
// the file names, the same on every machine.
#[test]
fn the_blocks_of_a_table_are_read_in_the_order_of_their_files() {
    let dir = TempDir::new();
    let (entities, _) = project(&dir);
    for (file, field) in [
        ("zebra.rs", "last"),
        ("alpha.rs", "first"),
        ("mango.rs", "middle"),
    ] {
        fs::write(
            format!("{entities}/{file}"),
            format!(r#"extend! {{ table: "eihwaz_users", fields: {{ {field}: text }} }}"#),
        )
        .unwrap();
    }

    let merged = merge_extend_schemas(scan_extend_blocks(&entities).unwrap());

    assert_eq!(merged.len(), 1);
    let columns: Vec<&str> = merged[0].columns.iter().map(|c| c.name.as_str()).collect();
    assert_eq!(columns, ["first", "middle", "last"]);
}

// The block is the only one of the project and it is gone: every column it
// added is to drop. Refused without `--force`, nothing written.
#[test]
fn a_removed_block_drops_its_columns_and_is_refused_without_force() {
    let dir = TempDir::new();
    let (entities, path) = extended_project(&dir, &[("profile.rs", PROFILE)]);
    assert!(
        dropped_by_extensions(&entities, &path).is_empty(),
        "applied"
    );

    fs::remove_file(format!("{entities}/profile.rs")).unwrap();
    let before = everything(&dir);

    assert_eq!(
        dropped_by_extensions(&entities, &path),
        [(
            "eihwaz_users".to_string(),
            vec!["bio".to_string(), "website".to_string()]
        )]
    );
    assert!(run(&entities, &path, false, DatabaseEngine::SQLite).is_err());
    assert_eq!(everything(&dir), before);
}

// With `--force`: one migration, the snapshot stays until it is applied,
// then goes; after that the project has nothing left to migrate.
#[test]
fn a_forced_block_removal_takes_its_snapshot_away_once_applied() {
    let dir = TempDir::new();
    let (entities, path) = extended_project(&dir, &[("profile.rs", PROFILE)]);
    let snapshot = extend_snapshot_file_path(&path, "eihwaz_users");
    assert!(Path::new(&snapshot).exists());
    fs::remove_file(format!("{entities}/profile.rs")).unwrap();

    run(&entities, &path, true, DatabaseEngine::SQLite).unwrap();

    let files = generated(&path);
    assert_eq!(files.len(), 1, "{files:?}");
    let removal = &files[0];
    assert!(removal.0.ends_with("_extend_eihwaz_users_table.rs"));
    assert!(has_marker(&removal.1, WAIT_MIGRATION));
    assert!(removal.1.contains("bio") && removal.1.contains("website"));
    assert!(Path::new(&snapshot).exists(), "not applied yet");

    // Run again before applying: the same removal, not a file moves.
    let waiting = everything(&dir);
    run(&entities, &path, true, DatabaseEngine::SQLite).unwrap();
    assert_eq!(everything(&dir), waiting);

    applied(&entities, &path);
    assert!(!Path::new(&snapshot).exists());
    assert!(!crate::atomic::interrupted(Path::new(&path)));

    let settled = everything(&dir);
    for force in [false, true] {
        run(&entities, &path, force, DatabaseEngine::SQLite).unwrap();
        assert_eq!(everything(&dir), settled, "force: {force}");
    }
}

// A block kept without a field is a block removed: same columns to drop,
// and no snapshot describing nothing once applied.
#[test]
fn a_block_left_without_a_field_is_treated_as_removed() {
    let dir = TempDir::new();
    let (entities, path) = extended_project(&dir, &[("profile.rs", PROFILE)]);
    fs::write(format!("{entities}/profile.rs"), PROFILE_EMPTY).unwrap();

    assert_eq!(
        dropped_by_extensions(&entities, &path),
        [(
            "eihwaz_users".to_string(),
            vec!["bio".to_string(), "website".to_string()]
        )]
    );
    assert!(run(&entities, &path, false, DatabaseEngine::SQLite).is_err());
    run(&entities, &path, true, DatabaseEngine::SQLite).unwrap();
    applied(&entities, &path);

    assert!(!Path::new(&extend_snapshot_file_path(&path, "eihwaz_users")).exists());
    let settled = everything(&dir);
    run(&entities, &path, false, DatabaseEngine::SQLite).unwrap();
    assert_eq!(everything(&dir), settled);
}

// The other side: a block that keeps a field keeps its snapshot, rewritten
// with what is left.
#[test]
fn a_block_that_keeps_a_field_keeps_its_snapshot() {
    let dir = TempDir::new();
    let (entities, path) = extended_project(&dir, &[("profile.rs", PROFILE)]);
    fs::write(format!("{entities}/profile.rs"), PROFILE_BIO).unwrap();

    assert_eq!(
        dropped_by_extensions(&entities, &path),
        [("eihwaz_users".to_string(), vec!["website".to_string()])]
    );
    run(&entities, &path, true, DatabaseEngine::SQLite).unwrap();
    applied(&entities, &path);

    let snapshot = fs::read_to_string(extend_snapshot_file_path(&path, "eihwaz_users")).unwrap();
    assert!(
        snapshot.contains("bio") && !snapshot.contains("website"),
        "{snapshot}"
    );
    assert!(dropped_by_extensions(&entities, &path).is_empty());
}

// Two tables extended, one block removed: only its columns go, and the
// other table keeps its block, its columns and its snapshot.
#[test]
fn only_the_removed_block_loses_its_columns() {
    let dir = TempDir::new();
    let (entities, path) = extended_project(&dir, &[("profile.rs", PROFILE), ("team.rs", TEAM)]);
    let team = fs::read_to_string(extend_snapshot_file_path(&path, "eihwaz_groupes")).unwrap();
    fs::remove_file(format!("{entities}/profile.rs")).unwrap();

    assert_eq!(
        dropped_by_extensions(&entities, &path),
        [(
            "eihwaz_users".to_string(),
            vec!["bio".to_string(), "website".to_string()]
        )]
    );
    run(&entities, &path, true, DatabaseEngine::SQLite).unwrap();
    applied(&entities, &path);

    assert!(!Path::new(&extend_snapshot_file_path(&path, "eihwaz_users")).exists());
    assert_eq!(
        fs::read_to_string(extend_snapshot_file_path(&path, "eihwaz_groupes")).unwrap(),
        team
    );
}

// The block put back before the removal is applied: the waiting migration
// goes, file and `lib.rs` entry, and the block and its snapshot are as they
// were. The `lib.rs` the removal created stays, without the migration.
#[test]
fn a_block_put_back_before_applying_cancels_its_removal() {
    let dir = TempDir::new();
    let (entities, path) = extended_project(&dir, &[("profile.rs", PROFILE)]);
    let settled = everything(&dir);
    fs::remove_file(format!("{entities}/profile.rs")).unwrap();
    run(&entities, &path, true, DatabaseEngine::SQLite).unwrap();
    assert_eq!(generated(&path).len(), 1);
    assert!(
        fs::read_to_string(lib_path(&path))
            .unwrap()
            .contains("_extend_eihwaz_users_table")
    );

    fs::write(format!("{entities}/profile.rs"), PROFILE).unwrap();
    run(&entities, &path, false, DatabaseEngine::SQLite).unwrap();

    assert!(generated(&path).is_empty());
    assert!(
        !fs::read_to_string(lib_path(&path))
            .unwrap()
            .contains("_extend_")
    );
    let left: Vec<(String, String)> = everything(&dir)
        .into_iter()
        .filter(|(file, _)| file != "src/lib.rs")
        .collect();
    assert_eq!(left, settled);
}

// ── An enum name declared by more than one model ─────────────────────────────

const ORDERS: &str = r#"model! { Order, table: "commandes", pk: id => i32, enums: { Statut: [Attente, Payee] },
    { reference: text [required], statut: choice [enum(Statut), required] } }"#;
const INVOICES: &str = r#"model! { Invoice, table: "factures", pk: id => i32, enums: { Statut: [Brouillon, Emise] },
    { numero: text [required], statut: choice [enum(Statut), required] } }"#;

/// What reading the models of `files` (file name, content) answers.
fn scanned(files: &[(&str, &str)]) -> Result<Vec<String>, String> {
    let dir = TempDir::new();
    let (entities, _) = project(&dir);
    for (name, content) in files {
        fs::write(format!("{entities}/{name}"), content).unwrap();
    }
    scan_models(&entities)
        .map(|found| found.schemas.into_iter().map(|s| s.table_name).collect())
        .map_err(|e| e.to_string())
}

// Two models, one enum name: refused, naming the enum and both tables,
// whether the values differ or not.
#[test]
fn an_enum_declared_by_two_models_is_refused() {
    let error = scanned(&[("order.rs", ORDERS), ("invoice.rs", INVOICES)]).unwrap_err();
    assert!(error.contains("  Statut: commandes, factures"), "{error}");

    let same_values = INVOICES.replace("[Brouillon, Emise]", "[Attente, Payee]");
    let error = scanned(&[("order.rs", ORDERS), ("invoice.rs", &same_values)]).unwrap_err();
    assert!(error.contains("  Statut: commandes, factures"), "{error}");
}

// The other side: each model its own name, and both are read.
#[test]
fn two_models_with_their_own_enum_names_are_read() {
    let renamed = INVOICES.replace("Statut", "StatutFacture");
    assert_eq!(
        scanned(&[("order.rs", ORDERS), ("invoice.rs", &renamed)]).unwrap(),
        ["factures", "commandes"]
    );
}

// One declaration used by several fields of its model, a list field
// included, is still one declaration.
#[test]
fn several_fields_of_a_model_share_its_enum() {
    let model = r#"model! { Order, table: "commandes", pk: id => i32, enums: { Statut: [Attente, Payee] },
        { statut: choice [enum(Statut), required], ancien: choice [enum(Statut), nullable],
          etapes: checkbox [enum(Statut)] } }"#;
    assert_eq!(
        scanned(&[("order.rs", model)]).unwrap(),
        ["commandes", "commandes_etapes"]
    );
}

// Every name declared twice is listed at once, sorted; a name declared once
// is not.
#[test]
fn every_enum_declared_twice_is_named_at_once() {
    let orders = r#"model! { Order, table: "commandes", pk: id => i32,
        enums: { Statut: [Attente, Payee], Canal: [Web, Boutique], Devise: [Eur, Usd] },
        { statut: choice [enum(Statut), required], canal: choice [enum(Canal), required],
          devise: choice [enum(Devise), required] } }"#;
    let invoices = r#"model! { Invoice, table: "factures", pk: id => i32,
        enums: { Statut: [Brouillon, Emise], Canal: [Web, Courrier] },
        { statut: choice [enum(Statut), required], canal: choice [enum(Canal), required] } }"#;

    let error = scanned(&[("order.rs", orders), ("invoice.rs", invoices)]).unwrap_err();

    let canal = error.find("  Canal: commandes, factures").expect(&error);
    let statut = error.find("  Statut: commandes, factures").expect(&error);
    assert!(canal < statut, "{error}");
    assert!(!error.contains("Devise"), "{error}");
}

// An enum stored as an integer is no type of the database: two models may
// each have one of the same name.
#[test]
fn an_integer_enum_is_not_a_database_type_and_may_be_declared_twice() {
    let orders = r#"model! { Order, table: "commandes", pk: id => i32, enums: { Priorite: i16 [Basse = 0, Haute = 1] },
        { priorite: choice [enum(Priorite), required] } }"#;
    let invoices = orders
        .replace("Order", "Invoice")
        .replace("commandes", "factures");
    assert_eq!(
        scanned(&[("order.rs", orders), ("invoice.rs", &invoices)]).unwrap(),
        ["factures", "commandes"]
    );
}

// A block extending a framework table declares its enums like a model.
#[test]
fn an_enum_of_an_extend_block_counts_as_a_declaration() {
    const BLOCK: &str = r#"extend! { table: "eihwaz_users", enums: { Statut: [Actif, Inactif] },
        fields: { statut: choice [enum(Statut), nullable] } }"#;

    let error = scanned(&[("order.rs", ORDERS), ("profile.rs", BLOCK)]).unwrap_err();
    assert!(
        error.contains("  Statut: commandes, extend!{} eihwaz_users"),
        "{error}"
    );

    let renamed = BLOCK.replace("Statut", "StatutCompte");
    assert_eq!(
        scanned(&[("order.rs", ORDERS), ("profile.rs", &renamed)]).unwrap(),
        ["commandes"]
    );
}

// A paused model is out of everything the command does, this check included.
#[test]
fn a_paused_model_is_out_of_the_enum_check() {
    let paused = INVOICES.replace(
        "statut: choice [enum(Statut), required] }",
        "statut: choice [enum(Statut), required] }, meta: { managed: false }",
    );
    assert_ne!(paused, INVOICES);
    assert_eq!(
        scanned(&[("order.rs", ORDERS), ("invoice.rs", &paused)]).unwrap(),
        ["commandes"]
    );
}

// The whole command: refused with or without `--force`, nothing written,
// not even the table of the model that is right.
#[test]
fn the_command_writes_nothing_when_an_enum_is_declared_twice() {
    let dir = TempDir::new();
    let (entities, path) = project(&dir);
    fs::write(format!("{entities}/user.rs"), USERS).unwrap();
    fs::write(format!("{entities}/order.rs"), ORDERS).unwrap();
    fs::write(format!("{entities}/invoice.rs"), INVOICES).unwrap();
    let before = everything(&dir);

    for force in [false, true] {
        let error = run(&entities, &path, force, DatabaseEngine::SQLite)
            .unwrap_err()
            .to_string();
        assert!(error.contains("  Statut: commandes, factures"), "{error}");
        assert_eq!(everything(&dir), before, "force: {force}");
    }

    // One of the two renamed: the three tables are migrated.
    fs::write(
        format!("{entities}/invoice.rs"),
        INVOICES.replace("Statut", "StatutFacture"),
    )
    .unwrap();
    run(&entities, &path, false, DatabaseEngine::SQLite).unwrap();
    assert_eq!(generated(&path).len(), 3);
}
