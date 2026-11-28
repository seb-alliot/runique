//! `migration up | down | status` — runs the project's own `migration` crate
//! (`cargo run -- up`), the way `sea-orm-cli migrate` does, without needing
//! `sea-orm-cli` installed: the SeaORM version that applies the migrations is
//! then the one `migration/Cargo.toml` pins, the same as runique's. Its
//! `down()` keeps `seaql_migrations` in sync with the schema.
use crate::{
    atomic::{AtomicBuilder, SourceItems},
    db::DatabaseConfig,
    migration::{
        ParsedEnum, ParsedSchema, WAIT_MIGRATION, dropped_table, enum_snapshot_file_path,
        enum_target, enums_of_schemas, extend_snapshot_file_path, generate_enum_snapshot_file,
        generate_snapshot_file, has_marker,
        master::BuilderMaster,
        migration_target, read_enum_snapshots, renamed_table, snapshot_file_path,
        steps::{
            apply::applied_renames::cleared_files,
            generate::lib_file::lib_modules,
            parser::files::{
                scan::{merge_extend_schemas, scan_entities, scan_extend_blocks},
                snapshot::{self, extend_snapshot_content},
            },
        },
        without_marker,
    },
};
use anyhow::{Context, Result};
use runique_i18n::{t, tf};
use std::{
    fs,
    path::{Path, PathBuf},
};

/// Applies every pending migration, then records what was applied
/// (see [`record_applied`]).
pub async fn up(entities_path: &str, migrations_path: &str) -> Result<()> {
    dotenvy::dotenv().ok();
    BuilderMaster::new()
        .migration_up(entities_path, migrations_path)
        .await?;
    println!("{}", t(runique_i18n::keys::migrate::info::COMPLETE_UP));
    Ok(())
}

/// After a run that succeeded, in one batch: every migration of `lib.rs`
/// still tagged as waiting loses its tag, and the table it changed gets its
/// snapshot written from its model. A model no applied migration concerns is
/// left without one: its table isn't created yet. An enum is recorded the
/// same way: the migration that created it or changed its values, applied,
/// gets its snapshot written from the models.
pub fn record_applied(entities_path: &str, migrations_path: &str) -> Result<()> {
    let mut files: Vec<(PathBuf, String)> = Vec::new();
    let mut targets: Vec<(bool, String)> = Vec::new();
    let mut enum_targets: Vec<String> = Vec::new();
    // Tables an applied migration dropped, and former names of the ones it
    // renamed: their snapshot goes.
    let mut gone: Vec<String> = Vec::new();
    for module in lib_modules(migrations_path)? {
        let path = Path::new(migrations_path).join(format!("{module}.rs"));
        if let Ok(content) = fs::read_to_string(&path)
            && has_marker(&content, WAIT_MIGRATION)
        {
            // A table renamed by this migration: the snapshot of its
            // former name goes, the new one is written below.
            if let Some(old) = renamed_table(&content) {
                gone.push(old.to_string());
            }
            files.push((path, without_marker(&content, WAIT_MIGRATION)));

            if let Some(table) = dropped_table(&module) {
                gone.push(table.to_string());
            } else if let Some((extend, table)) = migration_target(&module) {
                if !targets.iter().any(|(e, t)| *e == extend && t == table) {
                    targets.push((extend, table.to_string()));
                }
            } else if let Some(name) = enum_target(&module)
                && !enum_targets.iter().any(|known| known == name)
            {
                enum_targets.push(name.to_string());
            }
        }
    }
    if files.is_empty() {
        return Ok(());
    }

    // Dropped in this very run: no snapshot is written for it, even when a
    // model of that name is back.
    targets.retain(|(extend, table)| *extend || !gone.contains(table));
    // The models are only read when a snapshot is to be written: a run that
    // drops the last tables of a project has none left to read.
    let (models, extends) = if targets.is_empty() && enum_targets.is_empty() {
        (Vec::new(), Vec::new())
    } else {
        (
            scan_entities(entities_path)?,
            merge_extend_schemas(scan_extend_blocks(entities_path)?),
        )
    };
    // The renames just applied are over: their `renamed_from` leaves the
    // entity files, in the same batch as the snapshots that record them.
    if !targets.is_empty() {
        files.extend(cleared_files(entities_path, &targets)?);
    }
    // The tables whose snapshot this batch writes from their model.
    let rewritten: Vec<String> = targets.iter().map(|(_, table)| table.clone()).collect();
    // Extensions whose block is gone or has no field left: their snapshot goes.
    let mut emptied: Vec<PathBuf> = Vec::new();
    for (extend, table) in targets {
        if extend {
            let snapshot = PathBuf::from(extend_snapshot_file_path(migrations_path, &table));
            match extends.iter().find(|s| s.table_name == table) {
                Some(schema) if !schema.columns.is_empty() => {
                    files.push((snapshot, extend_snapshot_content(schema)));
                }
                _ => emptied.push(snapshot),
            }
        } else if let Some(schema) = models.iter().find(|s| s.table_name == table) {
            files.push((
                snapshot_file_path(migrations_path, &table).into(),
                generate_snapshot_file(schema),
            ));
        }
    }

    // The enums the applied migrations created or changed, as the models
    // declare them now. One no model declares any more has no snapshot to
    // write: it goes below, with the last column that used it.
    if !enum_targets.is_empty() {
        let declared: Vec<ParsedSchema> = models.iter().chain(&extends).cloned().collect();
        let applied: Vec<ParsedEnum> = enums_of_schemas(&declared)
            .into_iter()
            .filter(|found| enum_targets.contains(&found.name))
            .collect();
        for found in &applied {
            files.push((
                enum_snapshot_file_path(migrations_path, &found.name).into(),
                generate_enum_snapshot_file(found),
            ));
        }
        files.extend(tables_following_their_enums(
            migrations_path,
            &applied,
            &declared,
            &rewritten,
            &gone,
        )?);
    }

    let mut batch = AtomicBuilder::modify()
        .journal(migrations_path)
        .beside(entities_path);
    for (path, content) in files {
        batch = batch.add(path, SourceItems::Bytes(content.into_bytes()), false)?;
    }
    gone.sort();
    gone.dedup();
    for table in gone {
        let snapshot = PathBuf::from(snapshot_file_path(migrations_path, &table));
        // Already gone (removed by hand): nothing to take away.
        if snapshot.exists() {
            batch = batch.add(snapshot, SourceItems::Remove, false)?;
        }
    }
    for snapshot in emptied {
        if snapshot.exists() {
            batch = batch.add(snapshot, SourceItems::Remove, false)?;
        }
    }
    batch.start()?;
    drop_unused_enum_snapshots(entities_path, migrations_path)
}

/// The snapshots of the tables `applied` enums are used by, when no migration
/// of their own rewrites them (`rewritten`) or takes them away (`gone`): as
/// (path, content), each with its enum columns as the enum is now, its name
/// and its values. A renamed enum has no migration of its tables: without
/// this, their snapshots would still name it as it was, and the rename would
/// be found again.
///
/// Only the enum columns change: the rest of a snapshot is written back as it
/// was read, never from the model, which may hold changes not migrated yet.
fn tables_following_their_enums(
    migrations_path: &str,
    applied: &[ParsedEnum],
    declared: &[ParsedSchema],
    rewritten: &[String],
    gone: &[String],
) -> Result<Vec<(PathBuf, String)>> {
    // (path, the snapshot as read then corrected, of an extension)
    let mut followed: Vec<(String, ParsedSchema, bool)> = Vec::new();
    for found in applied {
        for place in &found.places {
            if rewritten.contains(&place.table) || gone.contains(&place.table) {
                continue;
            }
            let of_model = snapshot_file_path(migrations_path, &place.table);
            let of_extension = extend_snapshot_file_path(migrations_path, &place.table);
            let (path, extension) = if Path::new(&of_model).exists() {
                (of_model, false)
            } else if Path::new(&of_extension).exists() {
                (of_extension, true)
            } else {
                continue;
            };
            // A snapshot that doesn't record lengths can't be written back
            // as it was read: `makemigrations` rewrites it first.
            if !snapshot::records_lengths(&path)? {
                continue;
            }
            let index = match followed.iter().position(|(known, ..)| *known == path) {
                Some(index) => index,
                None => {
                    followed.push((path.clone(), snapshot::parse_create_file(&path)?, extension));
                    followed.len() - 1
                }
            };
            // The name as the model writes it, which the snapshot keeps.
            let declared_name = declared
                .iter()
                .find(|schema| schema.table_name == place.table)
                .and_then(|schema| schema.columns.iter().find(|c| c.name == place.column))
                .and_then(|column| column.enum_name.clone());
            if let Some(column) = followed[index]
                .1
                .columns
                .iter_mut()
                .find(|column| column.name == place.column)
            {
                column.enum_name = declared_name;
                column.enum_string_values = found.values.clone();
            }
        }
    }
    Ok(followed
        .into_iter()
        .map(|(path, schema, extension)| {
            let content = if extension {
                extend_snapshot_content(&schema)
            } else {
                generate_snapshot_file(&schema)
            };
            (PathBuf::from(path), content)
        })
        .collect())
}

/// Once the snapshots of the tables are written: the snapshot of an enum no
/// table uses any more is taken away. Its type went with the last column
/// that used it, and the same enum declared again later is a new one.
fn drop_unused_enum_snapshots(entities_path: &str, migrations_path: &str) -> Result<()> {
    let recorded = read_enum_snapshots(migrations_path)?;
    if recorded.is_empty() {
        return Ok(());
    }
    let tables: Vec<ParsedSchema> = snapshot::read_all(migrations_path)?
        .into_iter()
        .chain(snapshot::read_extend(migrations_path)?)
        .collect();
    let used = enums_of_schemas(&tables);
    let unused: Vec<&ParsedEnum> = recorded
        .iter()
        .filter(|enum_snapshot| !used.iter().any(|known| known.name == enum_snapshot.name))
        .collect();
    if unused.is_empty() {
        return Ok(());
    }
    let mut batch = AtomicBuilder::modify()
        .journal(migrations_path)
        .beside(entities_path);
    for gone in unused {
        batch = batch.add(
            PathBuf::from(enum_snapshot_file_path(migrations_path, &gone.name)),
            SourceItems::Remove,
            false,
        )?;
    }
    batch.start()?;
    Ok(())
}

/// Rolls back the last `n` applied migrations.
pub async fn down(migrations_path: &str, n: u32) -> Result<()> {
    dotenvy::dotenv().ok();
    BuilderMaster::new()
        .migration_down(migrations_path, n)
        .await?;
    println!("{}", t(runique_i18n::keys::migrate::info::COMPLETE_DOWN));
    Ok(())
}

/// Lists the migrations and whether each one is applied.
pub async fn status(migrations_path: &str) -> Result<()> {
    dotenvy::dotenv().ok();
    BuilderMaster::new().migration_status(migrations_path).await
}

/// The configuration of the project's database, by the app's own rule
/// ([`DatabaseConfig`]); an error when what the project sets is not enough.
pub(crate) fn configuration(get: impl Fn(&str) -> Option<String>) -> Result<DatabaseConfig> {
    let mut config = DatabaseConfig::from_lookup(get)
        .map_err(anyhow::Error::msg)?
        .build();
    // A command only ever needs one connection.
    config.pool(1, 1);
    Ok(config)
}

/// The folder of the migration crate behind `migrations_path`: its trailing
/// `/src` stripped.
fn crate_dir(migrations_path: &str) -> &str {
    migrations_path
        .trim_end_matches("/src")
        .trim_end_matches("\\src")
}

/// Refuses what cannot run: a path with no migration crate behind it, or a
/// crate a `makemigrations` cut short left half written.
pub(crate) fn runnable(migrations_path: &str) -> Result<()> {
    let migration_dir = crate_dir(migrations_path);
    // Checked first: cargo's own error for a wrong path is a raw "manifest
    // path does not exist".
    if !Path::new(migration_dir).join("Cargo.toml").is_file() {
        anyhow::bail!(
            "{}",
            tf(
                runique_i18n::keys::migrate::error::CRATE_MISSING,
                &[migration_dir]
            )
        );
    }

    // A `makemigrations` the power cut short left files it will take back:
    // nothing of them is applied. Refused, not repaired: this command writes
    // no file, the one that wrote them puts them right.
    for dir in [migration_dir, migrations_path] {
        if crate::atomic::interrupted(Path::new(dir)) {
            anyhow::bail!(
                "{}",
                tf(runique_i18n::keys::migrate::error::INTERRUPTED, &[dir])
            );
        }
    }
    Ok(())
}

/// `cargo run` on the migration crate behind `migrations_path`, from the
/// project root: a relative SQLite path resolves as the app's. `config` is
/// the database of the project, as the master read it.
pub(crate) async fn launch(
    migrations_path: &str,
    args: &[&str],
    config: &DatabaseConfig,
) -> Result<()> {
    let migration_dir = crate_dir(migrations_path);
    println!(
        "{}",
        tf(
            runique_i18n::keys::migrate::info::RUNNING,
            &[args[0], migration_dir]
        )
    );

    let mut command = tokio::process::Command::new("cargo");
    command
        .arg("run")
        .arg("--manifest-path")
        .arg(Path::new(migration_dir).join("Cargo.toml"))
        .arg("--")
        .args(args);
    // The crate reads `DATABASE_URL` and nothing else, while the app also
    // takes the `DB_*` keys and falls back on them: the address that answers
    // is found here and handed over, so both use the same database.
    // Through the crate's environment, never as an argument: the URL holds the
    // password, and arguments can be read by every user of the machine.
    let (_, answered) = config.connect_resolved().await.map_err(|e| {
        anyhow::anyhow!(tf(
            runique_i18n::keys::makemigrations::error::DATABASE_UNREACHABLE,
            &[e]
        ))
    })?;
    command.env("DATABASE_URL", &answered.url);
    let status = command
        .status()
        .await
        .with_context(|| t(runique_i18n::keys::migrate::error::CARGO_MISSING).into_owned())?;

    if !status.success() {
        let code = format!("{:?}", status.code());
        anyhow::bail!(
            "{}",
            tf(
                runique_i18n::keys::migrate::error::FAILED,
                &[args[0], code.as_str()]
            )
        );
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::atomic::tests::{TempDir, cut_journal};
    use std::fs;

    /// A migration crate under `dir`; the path of its `src`.
    fn migration_crate(dir: &TempDir) -> String {
        fs::create_dir_all(dir.join("migration/src")).unwrap();
        fs::write(dir.join("migration/Cargo.toml"), "[package]").unwrap();
        dir.join("migration/src").to_str().unwrap().to_string()
    }

    // A `makemigrations` cut short left its journal: whatever the command,
    // nothing is run, and the message says which folder to put right.
    #[tokio::test]
    async fn no_migration_is_applied_while_a_run_waits_to_be_repaired() {
        let dir = TempDir::new();
        let path = migration_crate(&dir);
        cut_journal(std::path::Path::new(&path), false, &[("lib.rs", true)], &[]);

        for outcome in [
            up("src/entities", &path).await,
            down(&path, 1).await,
            status(&path).await,
        ] {
            assert_eq!(
                outcome.unwrap_err().to_string(),
                tf(
                    runique_i18n::keys::migrate::error::INTERRUPTED,
                    &[path.as_str()]
                )
            );
        }
    }

    // The same when it is the crate itself that was being written.
    #[tokio::test]
    async fn no_migration_is_applied_while_the_crate_waits_to_be_repaired() {
        let dir = TempDir::new();
        let path = migration_crate(&dir);
        let crate_dir = dir.join("migration");
        cut_journal(&crate_dir, false, &[("src/lib.rs", false)], &[]);

        assert_eq!(
            up("src/entities", &path).await.unwrap_err().to_string(),
            tf(
                runique_i18n::keys::migrate::error::INTERRUPTED,
                &[crate_dir.to_str().unwrap()]
            )
        );
    }

    // ── What `up` records once it succeeded: tags cleared, snapshots written ──

    /// An entities folder that doesn't exist: reading it is an error.
    fn missing_entities(dir: &TempDir) -> String {
        dir.join("no_entities").to_str().unwrap().to_string()
    }

    const BODY: &str =
        "use sea_orm_migration::prelude::*;\n// runique: waiting migrate up, quoted\n";

    /// A `migration/src` whose `lib.rs` declares `modules`, with `files`.
    fn migrations(dir: &TempDir, modules: &[&str], files: &[(&str, String)]) -> String {
        let src = dir.join("src");
        fs::create_dir_all(&src).unwrap();
        let mods: String = modules.iter().map(|m| format!("mod {m};\n")).collect();
        fs::write(src.join("lib.rs"), mods).unwrap();
        for (name, content) in files {
            fs::write(src.join(name), content).unwrap();
        }
        src.to_str().unwrap().to_string()
    }

    fn tagged() -> String {
        format!("{WAIT_MIGRATION}\n{BODY}")
    }

    // Only the first line goes: the rest of the file is the same, byte for
    // byte, even where it quotes the tag.
    #[test]
    fn a_waiting_migration_loses_its_tag_and_nothing_else() {
        let dir = TempDir::new();
        let path = migrations(
            &dir,
            &["m1_create_a_table", "m2_alter_a_table"],
            &[
                ("m1_create_a_table.rs", tagged()),
                ("m2_alter_a_table.rs", tagged()),
            ],
        );

        // Neither name is one Runique gives: no snapshot is asked for.
        record_applied(&entities(&dir, &[]), &path).unwrap();

        for name in ["m1_create_a_table.rs", "m2_alter_a_table.rs"] {
            assert_eq!(
                fs::read_to_string(dir.join("src").join(name)).unwrap(),
                BODY
            );
        }
        // The batch is over: no journal is left to stop the next command.
        assert!(!crate::atomic::interrupted(Path::new(&path)));
    }

    // Untagged, quoting the tag inside a longer line (as `BODY` does),
    // declared without a file, or tagged but unknown to `lib.rs`: nothing is
    // rewritten, and the models aren't even read.
    #[test]
    fn only_tagged_migrations_of_lib_are_rewritten() {
        let dir = TempDir::new();
        let path = migrations(
            &dir,
            &["m1_applied", "m3_no_file"],
            &[
                ("m1_applied.rs", BODY.to_string()),
                ("m9_not_in_lib.rs", tagged()),
            ],
        );
        let on_disk = dir.entries();

        record_applied(&missing_entities(&dir), &path).unwrap();

        assert_eq!(dir.entries(), on_disk);
        let read = |name: &str| fs::read_to_string(dir.join("src").join(name)).unwrap();
        assert_eq!(read("m1_applied.rs"), BODY);
        assert_eq!(read("m9_not_in_lib.rs"), tagged());
    }

    // The tag is found wherever the developer moved it: below a header, in
    // the middle, on the last line with or without its line break, indented.
    #[test]
    fn the_tag_is_cleared_wherever_it_is() {
        let moved = [
            format!("// header\n{WAIT_MIGRATION}\n{BODY}"),
            format!("{BODY}{WAIT_MIGRATION}\n{BODY}"),
            format!("// header\n{BODY}{WAIT_MIGRATION}\n"),
            format!("// header\n{BODY}{WAIT_MIGRATION}"),
            format!("// header\n  {WAIT_MIGRATION}\t\r\n{BODY}"),
        ];
        let cleared = [
            format!("// header\n{BODY}"),
            format!("{BODY}{BODY}"),
            format!("// header\n{BODY}"),
            format!("// header\n{BODY}"),
            format!("// header\n{BODY}"),
        ];
        for (content, expected) in moved.into_iter().zip(cleared) {
            let dir = TempDir::new();
            let path = migrations(&dir, &["m1_moved"], &[("m1_moved.rs", content.clone())]);

            record_applied(&entities(&dir, &[]), &path).unwrap();

            assert_eq!(
                fs::read_to_string(dir.join("src/m1_moved.rs")).unwrap(),
                expected,
                "{content:?}"
            );
        }
    }

    // No `lib.rs`: nothing to clear, and no error.
    #[test]
    fn without_a_lib_nothing_is_cleared() {
        let dir = TempDir::new();
        let path = dir.join("src").to_str().unwrap().to_string();
        record_applied(&missing_entities(&dir), &path).unwrap();
        assert!(dir.entries().is_empty());
    }

    const MENUS: &str =
        r#"model! { Menu, table: "menus", pk: id => i32, { titre: text [required], } }"#;
    const PLATS: &str =
        r#"model! { Plat, table: "plats", pk: id => i32, { nom: text [required], } }"#;
    const EXTEND: &str = r#"extend! { table: "eihwaz_users", fields: { bio: text } }"#;

    /// An entities folder holding `files`.
    fn entities(dir: &TempDir, files: &[(&str, &str)]) -> String {
        let folder = dir.join("entities");
        fs::create_dir_all(&folder).unwrap();
        for (name, content) in files {
            fs::write(folder.join(name), content).unwrap();
        }
        folder.to_str().unwrap().to_string()
    }

    // The table of an applied migration gets the snapshot its model gives; a
    // model nothing was applied for gets none, or its table would never be
    // created.
    #[test]
    fn an_applied_table_gets_its_snapshot_and_no_other_does() {
        let dir = TempDir::new();
        let path = migrations(
            &dir,
            &["m20260101_000000_create_menus_table"],
            &[("m20260101_000000_create_menus_table.rs", tagged())],
        );
        let models = entities(&dir, &[("menus.rs", MENUS), ("plats.rs", PLATS)]);

        record_applied(&models, &path).unwrap();

        let menus = scan_entities(&models)
            .unwrap()
            .into_iter()
            .find(|s| s.table_name == "menus")
            .unwrap();
        assert_eq!(
            fs::read_to_string(snapshot_file_path(&path, "menus")).unwrap(),
            generate_snapshot_file(&menus)
        );
        assert!(!Path::new(&snapshot_file_path(&path, "plats")).exists());
        assert_eq!(
            fs::read_to_string(dir.join("src/m20260101_000000_create_menus_table.rs")).unwrap(),
            BODY
        );
    }

    // An `extend!{}` migration writes the extension's snapshot, in its own
    // folder, and never one at the root.
    #[test]
    fn an_applied_extension_gets_its_snapshot_in_its_folder() {
        let dir = TempDir::new();
        let path = migrations(
            &dir,
            &["m20260101_000000_extend_eihwaz_users_table"],
            &[("m20260101_000000_extend_eihwaz_users_table.rs", tagged())],
        );
        let models = entities(&dir, &[("ext.rs", EXTEND)]);

        record_applied(&models, &path).unwrap();

        let extension = merge_extend_schemas(scan_extend_blocks(&models).unwrap()).remove(0);
        assert_eq!(
            fs::read_to_string(extend_snapshot_file_path(&path, "eihwaz_users")).unwrap(),
            extend_snapshot_content(&extension)
        );
        assert!(!Path::new(&snapshot_file_path(&path, "eihwaz_users")).exists());
    }

    // Created then altered in the same run: one snapshot, written once.
    #[test]
    fn two_applied_migrations_of_a_table_write_one_snapshot() {
        let dir = TempDir::new();
        let path = migrations(
            &dir,
            &[
                "m20260101_000000_create_menus_table",
                "m20260102_000000_alter_menus_table",
            ],
            &[
                ("m20260101_000000_create_menus_table.rs", tagged()),
                ("m20260102_000000_alter_menus_table.rs", tagged()),
            ],
        );
        let models = entities(&dir, &[("menus.rs", MENUS)]);

        record_applied(&models, &path).unwrap();

        assert!(Path::new(&snapshot_file_path(&path, "menus")).exists());
        assert!(!crate::atomic::interrupted(Path::new(&path)));
    }

    // Two tables changed in the same run: each gets its own snapshot. Only the
    // same table, of the same kind, counts once.
    #[test]
    fn two_applied_tables_each_get_their_snapshot() {
        let dir = TempDir::new();
        let path = migrations(
            &dir,
            &[
                "m20260101_000000_create_menus_table",
                "m20260101_000000_create_plats_table",
            ],
            &[
                ("m20260101_000000_create_menus_table.rs", tagged()),
                ("m20260101_000000_create_plats_table.rs", tagged()),
            ],
        );
        let models = entities(&dir, &[("menus.rs", MENUS), ("plats.rs", PLATS)]);

        record_applied(&models, &path).unwrap();

        assert!(Path::new(&snapshot_file_path(&path, "menus")).exists());
        assert!(Path::new(&snapshot_file_path(&path, "plats")).exists());
    }

    // The model is gone since: the tag still goes, no snapshot is written.
    #[test]
    fn an_applied_table_without_a_model_gets_no_snapshot() {
        let dir = TempDir::new();
        let path = migrations(
            &dir,
            &["m20260101_000000_create_menus_table"],
            &[("m20260101_000000_create_menus_table.rs", tagged())],
        );
        let models = entities(&dir, &[("plats.rs", PLATS)]);

        record_applied(&models, &path).unwrap();

        assert!(!Path::new(&crate::migration::snapshot_dir(&path)).exists());
        assert_eq!(
            fs::read_to_string(dir.join("src/m20260101_000000_create_menus_table.rs")).unwrap(),
            BODY
        );
    }

    // Something was applied and the models can't be read: an error, and the
    // tags stay, so the next run records it.
    #[test]
    fn unreadable_models_leave_the_tags_in_place() {
        let dir = TempDir::new();
        let path = migrations(
            &dir,
            &["m20260101_000000_create_menus_table"],
            &[("m20260101_000000_create_menus_table.rs", tagged())],
        );

        assert!(record_applied(&missing_entities(&dir), &path).is_err());
        assert_eq!(
            fs::read_to_string(dir.join("src/m20260101_000000_create_menus_table.rs")).unwrap(),
            tagged()
        );
    }

    // ── The `renamed_from` just applied leaves the entity file ───────────────

    const BAKERY: &str = "model! {\n    Shop,\n    table: \"bakery\",\n    renamed_from: \"shop\",\n    pk: id => i32,\n    { title: text [required], }\n}\n";
    const BAKERY_CLEARED: &str = "model! {\n    Shop,\n    table: \"bakery\",\n    pk: id => i32,\n    { title: text [required], }\n}\n";

    /// A migration that renamed `shop`, still waiting.
    fn renaming() -> String {
        format!(
            "{WAIT_MIGRATION}\n{}shop\n{BODY}",
            crate::migration::RENAMED_TABLE
        )
    }

    // Applied: the directive goes from the model, the snapshot is under the
    // new name, the one of the former name is removed. One batch, no journal
    // left, and no other file in the entities folder.
    #[test]
    fn an_applied_rename_leaves_the_model_file() {
        let dir = TempDir::new();
        let path = migrations(
            &dir,
            &["m20260101_000000_alter_bakery_table"],
            &[("m20260101_000000_alter_bakery_table.rs", renaming())],
        );
        let models = entities(&dir, &[("shop.rs", BAKERY), ("plats.rs", PLATS)]);
        fs::create_dir_all(crate::migration::snapshot_dir(&path)).unwrap();
        fs::write(snapshot_file_path(&path, "shop"), "old").unwrap();

        record_applied(&models, &path).unwrap();

        let read = |name: &str| fs::read_to_string(dir.join("entities").join(name)).unwrap();
        assert_eq!(read("shop.rs"), BAKERY_CLEARED);
        assert_eq!(read("plats.rs"), PLATS);
        assert!(Path::new(&snapshot_file_path(&path, "bakery")).exists());
        assert!(!Path::new(&snapshot_file_path(&path, "shop")).exists());
        assert!(!crate::atomic::interrupted(Path::new(&path)));
        let left: Vec<String> = dir
            .entries()
            .into_iter()
            .filter(|entry| entry.starts_with("entities/"))
            .collect();
        assert_eq!(left, ["entities/plats.rs", "entities/shop.rs"]);
    }

    // A directive of a table nothing was applied for stays where it is.
    #[test]
    fn a_rename_not_applied_yet_stays_in_the_model_file() {
        let dir = TempDir::new();
        let path = migrations(
            &dir,
            &["m20260101_000000_create_plats_table"],
            &[("m20260101_000000_create_plats_table.rs", tagged())],
        );
        let models = entities(&dir, &[("shop.rs", BAKERY), ("plats.rs", PLATS)]);

        record_applied(&models, &path).unwrap();

        assert_eq!(
            fs::read_to_string(dir.join("entities/shop.rs")).unwrap(),
            BAKERY
        );
        assert!(Path::new(&snapshot_file_path(&path, "plats")).exists());
    }

    // The column directive of an extension goes the same way.
    #[test]
    fn an_applied_column_rename_leaves_the_extension_file() {
        let dir = TempDir::new();
        let path = migrations(
            &dir,
            &["m20260101_000000_extend_eihwaz_users_table"],
            &[("m20260101_000000_extend_eihwaz_users_table.rs", tagged())],
        );
        let models = entities(
            &dir,
            &[(
                "ext.rs",
                r#"extend! { table: "eihwaz_users", fields: { bio: text [renamed_from: "about"] } }"#,
            )],
        );

        record_applied(&models, &path).unwrap();

        assert_eq!(
            fs::read_to_string(dir.join("entities/ext.rs")).unwrap(),
            r#"extend! { table: "eihwaz_users", fields: { bio: text [] } }"#
        );
    }

    // The batch is one: when a snapshot can't be written, the model file
    // keeps its directive and the migration its tag.
    #[test]
    fn a_batch_that_fails_leaves_the_model_file_as_it_was() {
        let dir = TempDir::new();
        let path = migrations(
            &dir,
            &["m20260101_000000_alter_bakery_table"],
            &[("m20260101_000000_alter_bakery_table.rs", renaming())],
        );
        let models = entities(&dir, &[("shop.rs", BAKERY)]);
        // A file where the folder of the snapshots is to be.
        fs::write(crate::migration::snapshot_dir(&path), "not a folder").unwrap();

        assert!(record_applied(&models, &path).is_err());

        assert_eq!(
            fs::read_to_string(dir.join("entities/shop.rs")).unwrap(),
            BAKERY
        );
        assert_eq!(
            fs::read_to_string(dir.join("src/m20260101_000000_alter_bakery_table.rs")).unwrap(),
            renaming()
        );
        assert!(!crate::atomic::interrupted(Path::new(&path)));
    }

    // `.env` asks for SQLite, the project is compiled for PostgreSQL:
    // whatever the command, the crate is not launched.
    #[tokio::test]
    async fn an_engine_the_project_is_not_compiled_for_launches_nothing() {
        let dir = TempDir::new();
        let path = migration_crate(&dir);
        let master = || {
            BuilderMaster::new()
                .env(|key| (key == "DATABASE_URL").then(|| "sqlite::memory:".to_string()))
                .cargo(|| {
                    Ok(serde_json::json!({
                        "packages": [{ "name": "runique", "id": "runique 4.1.0" }],
                        "resolve": { "nodes": [{ "id": "runique 4.1.0", "features": ["postgres"] }] },
                    })
                    .to_string())
                })
        };

        for outcome in [
            master().migration_up("src/entities", &path).await,
            master().migration_down(&path, 1).await,
            master().migration_status(&path).await,
        ] {
            assert_eq!(
                outcome.unwrap_err().to_string(),
                tf(
                    runique_i18n::keys::makemigrations::error::ENGINE_NOT_IN_FEATURES,
                    &["SQLite", "postgres", "SQLite", "sqlite"]
                )
            );
        }
    }

    /// The address `configuration` retains for a project's environment.
    fn url_for(vars: &[(&str, &str)]) -> Result<Option<String>> {
        configuration(|key| {
            vars.iter()
                .find(|(name, _)| *name == key)
                .map(|(_, value)| value.to_string())
        })
        .map(|config| Some(config.url))
    }

    // The project says where its database is: that address comes first, the
    // keys only stand by.
    #[test]
    fn a_database_url_the_project_sets_comes_first() {
        let set = url_for(&[
            ("DATABASE_URL", "postgres://app:secret@db.local/shop"),
            ("DB_ENGINE", "sqlite"),
        ]);
        assert_eq!(
            set.unwrap().as_deref(),
            Some("postgres://app:secret@db.local/shop")
        );
    }

    // Configured the other way the app accepts, by its parts: the crate gets
    // the URL the app itself would connect to.
    #[test]
    fn a_database_set_by_its_parts_gets_the_url_the_app_builds() {
        let parts = [
            ("DB_ENGINE", "Postgres"),
            ("DB_USER", "app"),
            ("DB_PASSWORD", "secret"),
            ("DB_NAME", "shop"),
        ];
        assert_eq!(
            url_for(&parts).unwrap().as_deref(),
            Some("postgres://app:secret@localhost:5432/shop")
        );
        assert_eq!(
            url_for(&[
                ("DB_ENGINE", "mysql"),
                ("DB_USER", "app"),
                ("DB_PASSWORD", "secret"),
                ("DB_HOST", "db.local"),
                ("DB_PORT", "3307"),
                ("DB_NAME", "shop"),
            ])
            .unwrap()
            .as_deref(),
            Some("mysql://app:secret@db.local:3307/shop")
        );
        assert_eq!(
            url_for(&parts).unwrap(),
            Some(
                DatabaseConfig::from_lookup(|key| {
                    parts
                        .iter()
                        .find(|(name, _)| *name == key)
                        .map(|(_, value)| value.to_string())
                })
                .unwrap()
                .build()
                .url
            ),
            "the very URL of the app"
        );
    }

    // Nothing set: SQLite and its default file, as for the app.
    #[test]
    fn a_project_that_sets_nothing_gets_the_default_sqlite_file() {
        assert_eq!(
            url_for(&[]).unwrap().as_deref(),
            Some("sqlite://local_base.sqlite?mode=rwc")
        );
        assert_eq!(
            url_for(&[("DB_ENGINE", "sqlite"), ("DB_NAME", "data.db")])
                .unwrap()
                .as_deref(),
            Some("sqlite://data.db?mode=rwc")
        );
    }

    // Parts that are not enough to connect: the command stops on the app's
    // own message, which names what is missing and never the password.
    #[test]
    fn parts_that_are_not_enough_stop_the_command() {
        let missing_user = url_for(&[("DB_ENGINE", "postgres"), ("DB_PASSWORD", "secret")])
            .unwrap_err()
            .to_string();
        assert!(missing_user.contains("DB_USER"), "{missing_user}");
        assert!(!missing_user.contains("secret"), "{missing_user}");

        let unknown = url_for(&[("DB_ENGINE", "oracle")]).unwrap_err().to_string();
        assert!(unknown.contains("oracle"), "{unknown}");
    }
}
