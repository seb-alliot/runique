//! `makemigrations`: the command's entry point. Its logic lives in
//! `migration::cli_migration`.
use anyhow::Result;

/// Connects to the database, then scans the models, diffs them against their
/// snapshots and writes the migration files
/// (see [`crate::migration::cli_migration::makemigrations`]).
pub async fn run(entities_path: &str, migrations_path: &str, force: bool) -> Result<()> {
    crate::migration::cli_migration::makemigrations(entities_path, migrations_path, force).await
}
