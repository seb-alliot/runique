use super::*;

/// Snapshots written before column lengths were recorded, for tables this
/// run doesn't otherwise touch: rewritten once with the model's lengths, so
/// later length changes are diffed instead of adopted forever. No migration.
pub(crate) fn snapshot_upgrades(
    entities_path: &str,
    schemas: &[ParsedSchema],
    main_changes: &[Changes],
    extend_planned: &[(ParsedSchema, Changes)],
    migrations_path: &str,
) -> Result<Vec<(String, String)>> {
    let mut upgrades = Vec::new();
    for schema in schemas {
        let path = snapshot_file_path(migrations_path, &schema.table_name);
        if Path::new(&path).exists()
            && !main_changes
                .iter()
                .any(|c| c.table_name == schema.table_name)
            && !records_lengths(&path)?
        {
            upgrades.push((path, generate_snapshot_file(schema)));
        }
    }
    for ext_schema in merge_extend_schemas(scan_extend_blocks(entities_path)?) {
        let path = extend_snapshot_file_path(migrations_path, &ext_schema.table_name);
        if Path::new(&path).exists()
            && !extend_planned
                .iter()
                .any(|(s, _)| s.table_name == ext_schema.table_name)
            && !records_lengths(&path)?
        {
            upgrades.push((path, extend_snapshot_content(&ext_schema)));
        }
    }
    Ok(upgrades)
}
