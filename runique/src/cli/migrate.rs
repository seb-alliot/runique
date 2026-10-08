//! `migration up` command — applies the SeaORM migrations through `sea-orm-cli`.
//! Rolling back and listing them are `sea-orm-cli migrate down` / `status`:
//! SeaORM runs each migration's `down()` and keeps `seaql_migrations` in sync.
use crate::utils::trad::{t, tf};
use anyhow::{Context, Result};

/// Runs `sea-orm-cli migrate up` against `migrations_path` (its trailing `/src`
/// stripped, since `sea-orm-cli` expects the migration crate root).
pub async fn up(migrations_path: &str) -> Result<()> {
    dotenvy::dotenv().ok();

    let migration_dir = migrations_path
        .trim_end_matches("/src")
        .trim_end_matches("\\src");

    // Checked before launching sea-orm-cli: its own error for a wrong path is
    // cargo's raw "manifest path does not exist".
    if !std::path::Path::new(migration_dir)
        .join("Cargo.toml")
        .is_file()
    {
        anyhow::bail!("{}", tf("migrate.crate_missing", &[migration_dir]));
    }

    println!("{}", tf("migrate.applying", &[migration_dir]));

    let status = tokio::process::Command::new("sea-orm-cli")
        .args(["migrate", "up", "--migration-dir", migration_dir])
        .status()
        .await
        .with_context(
            || "Unable to launch sea-orm-cli. Is it installed? Run: cargo install sea-orm-cli",
        )?;

    if !status.success() {
        anyhow::bail!("sea-orm-cli migrate up failed (code: {:?})", status.code());
    }

    println!("{}", t("migrate.complete_up"));
    Ok(())
}
