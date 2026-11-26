use super::*;

/// Computes the diff for every scanned model (no writing). A table of
/// `renames` (former name, name) is compared to the snapshot of its former
/// name; what the renames explain is taken out of every diff and kept in
/// `Changes::renames`.
pub(crate) fn compute_main_changes(
    schemas: &[ParsedSchema],
    renames: &[(String, String)],
    migrations_path: &str,
) -> Result<Vec<Changes>> {
    let mut all_changes: Vec<Changes> = Vec::new();
    let mut changed_keys: Vec<String> = Vec::new();

    for schema in schemas {
        let snapshot_of = renames
            .iter()
            .find(|(_, new)| *new == schema.table_name)
            .map_or(&schema.table_name, |(old, _)| old);

        let snap_path = snapshot_file_path(migrations_path, snapshot_of);

        let changes = if Path::new(&snap_path).exists() {
            let mut previous = previous_snapshot(&snap_path, schema)?;

            changed_keys.extend(primary_key_change(&previous, schema));

            let followed = follow_renames(&mut previous, schema, renames);
            Changes {
                renames: followed,
                ..diff_schemas(&previous, schema)
            }
        } else {
            creation_of(schema)
        };
        if !changes.is_empty() {
            all_changes.push(changes);
        }
    }
    if !changed_keys.is_empty() {
        anyhow::bail!(
            "{}",
            tf(
                runique_i18n::keys::makemigrations::error::PRIMARY_KEY_CHANGED,
                &[changed_keys.join("\n")]
            )
        );
    }
    Ok(all_changes)
}
