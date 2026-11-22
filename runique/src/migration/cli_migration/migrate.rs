//! `migration up | down | status` — runs the project's own `migration` crate
//! (`cargo run -- up`), the way `sea-orm-cli migrate` does, without needing
//! `sea-orm-cli` installed: the SeaORM version that applies the migrations is
//! then the one `migration/Cargo.toml` pins, the same as runique's. Its
//! `down()` keeps `seaql_migrations` in sync with the schema.
use super::lib_file::lib_modules;
use super::scan::{merge_extend_schemas, scan_entities, scan_extend_blocks};
use super::snapshot::extend_snapshot_content;
use crate::atomic::{AtomicBuilder, SourceItems};
use crate::db::DatabaseConfig;
use crate::migration::{
    WAIT_MIGRATION, dropped_table, extend_snapshot_file_path, generate_snapshot_file,
    has_marker, migration_target, renamed_table, snapshot_file_path, without_marker,
};
use crate::utils::trad::{t, tf};
use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};

/// Applies every pending migration, then records what was applied
/// (see [`record_applied`]).
pub async fn up(entities_path: &str, migrations_path: &str) -> Result<()> {
    run(migrations_path, &["up"]).await?;
    record_applied(entities_path, migrations_path)?;
    println!("{}", t("migrate.complete_up"));
    Ok(())
}

/// After a run that succeeded, in one batch: every migration of `lib.rs`
/// still tagged as waiting loses its tag, and the table it changed gets its
/// snapshot written from its model. A model no applied migration concerns is
/// left without one: its table isn't created yet.
pub fn record_applied(entities_path: &str, migrations_path: &str) -> Result<()> {
    let mut files: Vec<(PathBuf, String)> = Vec::new();
    let mut targets: Vec<(bool, String)> = Vec::new();
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
            } else if let Some((extend, table)) = migration_target(&module)
                && !targets.iter().any(|(e, t)| *e == extend && t == table)
            {
                targets.push((extend, table.to_string()));
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
    let (models, extends) = if targets.is_empty() {
        (Vec::new(), Vec::new())
    } else {
        (
            scan_entities(entities_path)?,
            merge_extend_schemas(scan_extend_blocks(entities_path)?),
        )
    };
    for (extend, table) in targets {
        if extend {
            if let Some(schema) = extends.iter().find(|s| s.table_name == table) {
                files.push((
                    extend_snapshot_file_path(migrations_path, &table).into(),
                    extend_snapshot_content(schema),
                ));
            }
        } else if let Some(schema) = models.iter().find(|s| s.table_name == table) {
            files.push((
                snapshot_file_path(migrations_path, &table).into(),
                generate_snapshot_file(schema),
            ));
        }
    }

    let mut batch = AtomicBuilder::modify().journal(migrations_path);
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
    batch.start()?;
    Ok(())
}

/// Rolls back the last `n` applied migrations.
pub async fn down(migrations_path: &str, n: u32) -> Result<()> {
    run(migrations_path, &["down", "-n", &n.to_string()]).await?;
    println!("{}", t("migrate.complete_down"));
    Ok(())
}

/// Lists the migrations and whether each one is applied.
pub async fn status(migrations_path: &str) -> Result<()> {
    run(migrations_path, &["status"]).await
}

/// The configuration of the project's database, by the app's own rule
/// ([`DatabaseConfig`]); an error when what the project sets is not enough.
fn configuration(get: impl Fn(&str) -> Option<String>) -> Result<DatabaseConfig> {
    let mut config = DatabaseConfig::from_lookup(get)
        .map_err(anyhow::Error::msg)?
        .build();
    // The command only checks which address answers.
    config.pool(1, 1);
    Ok(config)
}

/// `cargo run` on the migration crate behind `migrations_path` (its trailing
/// `/src` stripped), from the project root: the crate reads `DATABASE_URL`
/// from the root `.env`, and a relative SQLite path resolves as the app's.
async fn run(migrations_path: &str, args: &[&str]) -> Result<()> {
    dotenvy::dotenv().ok();

    let migration_dir = migrations_path
        .trim_end_matches("/src")
        .trim_end_matches("\\src");
    let manifest = std::path::Path::new(migration_dir).join("Cargo.toml");

    // Checked first: cargo's own error for a wrong path is a raw "manifest
    // path does not exist".
    if !manifest.is_file() {
        anyhow::bail!("{}", tf("migrate.crate_missing", &[migration_dir]));
    }

    // A `makemigrations` the power cut short left files it will take back:
    // nothing of them is applied. Refused, not repaired: this command writes
    // no file, the one that wrote them puts them right.
    for dir in [migration_dir, migrations_path] {
        if crate::atomic::interrupted(std::path::Path::new(dir)) {
            anyhow::bail!("{}", tf("migrate.interrupted", &[dir]));
        }
    }

    println!("{}", tf("migrate.running", &[args[0], migration_dir]));

    let mut command = tokio::process::Command::new("cargo");
    command
        .arg("run")
        .arg("--manifest-path")
        .arg(&manifest)
        .arg("--")
        .args(args);
    // The crate reads `DATABASE_URL` and nothing else, while the app also
    // takes the `DB_*` keys and falls back on them: the address that answers
    // is found here and handed over, so both use the same database.
    // Through the crate's environment, never as an argument: the URL holds the
    // password, and arguments can be read by every user of the machine.
    let config = configuration(|key| std::env::var(key).ok())?;
    let (connection, answered) = config
        .connect_resolved()
        .await
        .map_err(|e| anyhow::anyhow!(tf("makemigrations.database_unreachable", &[e])))?;
    drop(connection);
    command.env("DATABASE_URL", &answered.url);
    let status = command
        .status()
        .await
        .with_context(|| t("migrate.cargo_missing").into_owned())?;

    if !status.success() {
        let code = format!("{:?}", status.code());
        anyhow::bail!("{}", tf("migrate.failed", &[args[0], code.as_str()]));
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
                tf("migrate.interrupted", &[path.as_str()])
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
            tf("migrate.interrupted", &[crate_dir.to_str().unwrap()])
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
