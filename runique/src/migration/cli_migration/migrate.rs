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
