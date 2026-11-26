use super::*;

/// The migrations of `lib.rs` that are not waiting, in order: what a new
/// migration can start from.
pub(crate) fn settled_migrations(
    migrations_path: &str,
    waiting: &[(String, String)],
) -> Result<Vec<String>> {
    Ok(lib_modules(migrations_path)?
        .into_iter()
        .filter(|module| !waiting.iter().any(|(w, _)| w == module))
        .collect())
}
