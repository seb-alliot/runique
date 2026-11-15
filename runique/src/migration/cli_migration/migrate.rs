//! `migration up | down | status` — runs the project's own `migration` crate
//! (`cargo run -- up`), the way `sea-orm-cli migrate` does, without needing
//! `sea-orm-cli` installed: the SeaORM version that applies the migrations is
//! then the one `migration/Cargo.toml` pins, the same as runique's. Its
//! `down()` keeps `seaql_migrations` in sync with the schema.
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

    let status = tokio::process::Command::new("cargo")
        .arg("run")
        .arg("--manifest-path")
        .arg(&manifest)
        .arg("--")
        .args(args)
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
}
