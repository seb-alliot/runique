//! Two migrations written in parallel for the same thing.
use crate::migration::{steps::generate::lib_file::lib_modules, *};
use anyhow::Result;
use runique_i18n::tf;
use std::fs;

/// Refuses two migrations of `lib.rs` that say they start from the same one
/// for the same table: they were written in parallel, and Runique can't tell
/// which one the table should follow. A migration that doesn't say where it
/// starts from is left out.
pub fn check_parallel(migrations_path: &str) -> Result<()> {
    // (what it is about, started from, module)
    let mut seen: Vec<(String, String, String)> = Vec::new();
    let mut clashes: Vec<String> = Vec::new();
    for module in lib_modules(migrations_path)? {
        let Some(target) = module_target(&module) else {
            continue;
        };
        let Ok(content) = fs::read_to_string(format!("{migrations_path}/{module}.rs")) else {
            continue;
        };
        let Some(from) = started_from(&content) else {
            continue;
        };
        // A table, an extension and an enum of one name are three things.
        let about = format!("{target:?}");
        if let Some((.., other)) = seen.iter().find(|(a, f, _)| *a == about && f == from) {
            clashes.push(format!("{}: {other}, {module}", target.name()));
        }
        seen.push((about, from.to_string(), module.clone()));
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
