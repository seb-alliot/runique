//! `makemigrations`: the command's entry point. Its logic lives in
//! `migration::cli_migration`.
use anyhow::Result;

/// Scans the models, diffs them against their snapshots and writes the
/// migration files (see [`crate::migration::cli_migration::run`]).
pub fn run(entities_path: &str, migrations_path: &str, force: bool) -> Result<()> {
    crate::migration::cli_migration::run(entities_path, migrations_path, force)
}
