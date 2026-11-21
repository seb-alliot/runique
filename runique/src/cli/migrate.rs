//! `migration up | down | status`: the command's entry points. Their logic
//! lives in `migration::cli_migration::migrate`.
use anyhow::Result;

/// Applies every pending migration, then records what was applied: tags
/// cleared, snapshots of the changed tables written from `entities_path`.
pub async fn up(entities_path: &str, migrations_path: &str) -> Result<()> {
    crate::migration::cli_migration::migrate::up(entities_path, migrations_path).await
}

/// Rolls back the last `n` applied migrations.
pub async fn down(migrations_path: &str, n: u32) -> Result<()> {
    crate::migration::cli_migration::migrate::down(migrations_path, n).await
}

/// Lists the migrations and whether each one is applied.
pub async fn status(migrations_path: &str) -> Result<()> {
    crate::migration::cli_migration::migrate::status(migrations_path).await
}
