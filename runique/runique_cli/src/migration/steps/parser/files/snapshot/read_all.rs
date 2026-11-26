use super::*;

/// Reads every snapshot of `snapshots/` and `snapshots/runique/`: each `.rs`
/// must parse and describe the table its file is named after. Any other file
/// is ignored. Returns the tables of `snapshots/` itself, the ones Runique
/// migrated for a model: the `extend!{}` snapshots are checked, not returned.
pub(crate) fn read_all(migrations_path: &str) -> Result<Vec<ParsedSchema>> {
    let mut found = Vec::new();
    for (dir, of_a_model) in [
        (snapshot_dir(migrations_path), true),
        (extend_snapshot_dir(migrations_path), false),
    ] {
        if !Path::new(&dir).is_dir() {
            continue;
        }
        for path in rust_files(Path::new(&dir))? {
            let shown = path.display().to_string();
            let schema = parse_create_file(&shown).map_err(|_| {
                anyhow::anyhow!(tf(
                    runique_i18n::keys::makemigrations::error::SNAPSHOT_UNREADABLE,
                    &[&shown]
                ))
            })?;
            let expected = path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or_default();
            if schema.table_name != expected {
                anyhow::bail!(
                    "{}",
                    tf(
                        runique_i18n::keys::makemigrations::error::SNAPSHOT_MISNAMED,
                        &[shown.as_str(), schema.table_name.as_str()]
                    )
                );
            }
            if of_a_model {
                found.push(schema);
            }
        }
    }
    Ok(found)
}
