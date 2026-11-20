//! `migration up | down | status` — runs the project's own `migration` crate
//! (`cargo run -- up`), the way `sea-orm-cli migrate` does, without needing
//! `sea-orm-cli` installed: the SeaORM version that applies the migrations is
//! then the one `migration/Cargo.toml` pins, the same as runique's. Its
//! `down()` keeps `seaql_migrations` in sync with the schema.
use super::lib_file::lib_modules;
use crate::atomic::{AtomicBuilder, SourceItems};
use crate::db::DatabaseConfig;
use crate::migration::WAIT_MIGRATION;
use crate::utils::trad::{t, tf};
use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

/// Applies every pending migration.
pub async fn up(migrations_path: &str) -> Result<()> {
    run(migrations_path, &["up"]).await?;
    clear_waiting_tags(migrations_path)?;
    println!("{}", t("migrate.complete_up"));
    Ok(())
}

/// After a run that succeeded: every migration of `lib.rs` still tagged as
/// waiting loses its tag, the rest of its file untouched. One batch: all the
/// files, or none.
fn clear_waiting_tags(migrations_path: &str) -> Result<()> {
    let tag = format!("{WAIT_MIGRATION}\n");
    let mut cleared = Vec::new();
    for module in lib_modules(migrations_path)? {
        let path = Path::new(migrations_path).join(format!("{module}.rs"));
        if let Ok(content) = fs::read_to_string(&path)
            && let Some(rest) = content.strip_prefix(&tag)
        {
            cleared.push((path, rest.to_string()));
        }
    }
    if cleared.is_empty() {
        return Ok(());
    }
    let mut batch = AtomicBuilder::modify().journal(migrations_path);
    for (path, content) in cleared {
        batch = batch.add(path, SourceItems::Bytes(content.into_bytes()), false)?;
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

/// The `DATABASE_URL` to hand the migration crate when the project sets none.
/// The crate reads nothing else, while the app also takes `DB_ENGINE` and the
/// `DB_*` variables: built by the app's own rule ([`DatabaseConfig`]), so
/// both connect to the same database. `None` when the project sets the URL
/// itself, an error when what it sets is not enough to connect.
fn database_url(get: impl Fn(&str) -> Option<String>) -> Result<Option<String>> {
    if get("DATABASE_URL").is_some() {
        return Ok(None);
    }
    let config = DatabaseConfig::from_lookup(get)
        .map_err(anyhow::Error::msg)?
        .build();
    Ok(Some(config.url))
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
    // Through the crate's environment, never as an argument: the URL holds the
    // password, and arguments can be read by every user of the machine.
    if let Some(url) = database_url(|key| std::env::var(key).ok())? {
        command.env("DATABASE_URL", url);
    }
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

        for outcome in [up(&path).await, down(&path, 1).await, status(&path).await] {
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
            up(&path).await.unwrap_err().to_string(),
            tf("migrate.interrupted", &[crate_dir.to_str().unwrap()])
        );
    }

    // ── The waiting tag, cleared once `up` succeeded ─────────────────────────

    const BODY: &str = "use sea_orm_migration::prelude::*;\n// runique: waiting migrate up, quoted\n";

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

        clear_waiting_tags(&path).unwrap();

        for name in ["m1_create_a_table.rs", "m2_alter_a_table.rs"] {
            assert_eq!(fs::read_to_string(dir.join("src").join(name)).unwrap(), BODY);
        }
        // The batch is over: no journal is left to stop the next command.
        assert!(!crate::atomic::interrupted(Path::new(&path)));
    }

    // Untagged, tagged elsewhere than on the first line, declared without a
    // file, or tagged but unknown to `lib.rs`: nothing is rewritten.
    #[test]
    fn only_tagged_migrations_of_lib_are_rewritten() {
        let dir = TempDir::new();
        let below = format!("// header\n{WAIT_MIGRATION}\n{BODY}");
        let path = migrations(
            &dir,
            &["m1_applied", "m2_tag_below", "m3_no_file"],
            &[
                ("m1_applied.rs", BODY.to_string()),
                ("m2_tag_below.rs", below.clone()),
                ("m9_not_in_lib.rs", tagged()),
            ],
        );
        let on_disk = dir.entries();

        clear_waiting_tags(&path).unwrap();

        assert_eq!(dir.entries(), on_disk);
        let read = |name: &str| fs::read_to_string(dir.join("src").join(name)).unwrap();
        assert_eq!(read("m1_applied.rs"), BODY);
        assert_eq!(read("m2_tag_below.rs"), below);
        assert_eq!(read("m9_not_in_lib.rs"), tagged());
    }

    // No `lib.rs`: nothing to clear, and no error.
    #[test]
    fn without_a_lib_nothing_is_cleared() {
        let dir = TempDir::new();
        let path = dir.join("src").to_str().unwrap().to_string();
        clear_waiting_tags(&path).unwrap();
        assert!(dir.entries().is_empty());
    }

    /// What `database_url` gives for a project's environment.
    fn url_for(vars: &[(&str, &str)]) -> Result<Option<String>> {
        database_url(|key| {
            vars.iter()
                .find(|(name, _)| *name == key)
                .map(|(_, value)| value.to_string())
        })
    }

    // The project says where its database is: nothing is worked out, the crate
    // reads that URL itself.
    #[test]
    fn a_database_url_the_project_sets_is_left_to_the_crate() {
        let set = url_for(&[
            ("DATABASE_URL", "postgres://app:secret@db.local/shop"),
            ("DB_ENGINE", "sqlite"),
        ]);
        assert_eq!(set.unwrap(), None);
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
