use super::*;

/// Refuses two migrations of `lib.rs` that say they start from the same one
/// for the same table: they were written in parallel, and Runique can't tell
/// which one the table should follow. A migration that doesn't say where it
/// starts from is left out.
pub(crate) fn check_parallel(migrations_path: &str) -> Result<()> {
    // (extend, table, started from, module)
    let mut seen: Vec<(bool, String, String, String)> = Vec::new();
    let mut clashes: Vec<String> = Vec::new();
    for module in lib_modules(migrations_path)? {
        let Some((extend, table)) = migration_target(&module) else {
            continue;
        };
        let Ok(content) = fs::read_to_string(format!("{migrations_path}/{module}.rs")) else {
            continue;
        };
        let Some(from) = started_from(&content) else {
            continue;
        };
        if let Some((.., other)) = seen
            .iter()
            .find(|(e, t, f, _)| *e == extend && t == table && f == from)
        {
            clashes.push(format!("{table}: {other}, {module}"));
        }
        seen.push((extend, table.to_string(), from.to_string(), module.clone()));
    }
    if clashes.is_empty() {
        return Ok(());
    }
    anyhow::bail!(
        "{}",
        tf(
            runique_i18n::keys::makemigrations::error::PARALLEL_CHANGES,
            &[clashes.join("\n  ")]
        )
    )
}
