//! `migration up | down | status` — runs the project's own `migration` crate
//! (`cargo run -- up`), the way `sea-orm-cli migrate` does, without needing
//! `sea-orm-cli` installed: the SeaORM version that applies the migrations is
//! then the one `migration/Cargo.toml` pins, the same as runique's. Its
//! `down()` keeps `seaql_migrations` in sync with the schema.
use crate::db::DatabaseConfig;
use crate::utils::trad::{t, tf};
use anyhow::{Context, Result};

/// Applies every pending migration.
pub async fn up(migrations_path: &str) -> Result<()> {
    run(migrations_path, &["up"]).await?;
    println!("{}", t("migrate.complete_up"));
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
