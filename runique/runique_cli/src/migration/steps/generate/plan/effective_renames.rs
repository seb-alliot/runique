use super::*;

/// The declared renames (`declared`: former table, table) that hold: the
/// former name has a snapshot and no model any more, and the new name has no
/// snapshot yet. Same guard as a column's `renamed_from`: once applied, or
/// when it never matched, the declaration does nothing. Two tables can't
/// both come from one.
pub(crate) fn effective_renames(
    declared: &[(String, String)],
    schemas: &[ParsedSchema],
    paused: &[String],
    migrations_path: &str,
) -> Result<Vec<(String, String)>> {
    let has_snapshot =
        |table: &str| Path::new(&snapshot_file_path(migrations_path, table)).exists();
    let mut renames: Vec<(String, String)> = Vec::new();
    for (old, new) in declared {
        if !has_snapshot(old)
            || has_snapshot(new)
            || paused.contains(old)
            || schemas.iter().any(|s| s.table_name == *old)
        {
            continue;
        }
        if let Some((_, other)) = renames.iter().find(|(o, _)| o == old) {
            anyhow::bail!(
                "{}",
                tf(
                    runique_i18n::keys::makemigrations::error::RENAMED_TWICE,
                    &[old, other, new]
                )
            );
        }
        renames.push((old.clone(), new.clone()));
    }
    Ok(renames)
}
