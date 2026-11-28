//! Every `extend!{}` block against its snapshot.
use crate::migration::{
    steps::parser::files::snapshot::{extended_tables, previous_snapshot},
    *,
};
use anyhow::Result;
use std::path::Path;

/// Scans + merges `extend!{}` blocks and computes their diffs (no writing).
/// Returns the owned schema + change pair for each table that actually changed.
pub fn plan_extend_changes(
    entities_path: &str,
    migrations_path: &str,
) -> Result<Vec<(ParsedSchema, Changes)>> {
    let mut extend_schemas = merge_extend_schemas(scan_extend_blocks(entities_path)?);
    // A block removed since its columns were applied leaves its snapshot
    // behind: compared as a block without a field, every column it added
    // becomes a column to drop, guarded like any other.
    for table in extended_tables(migrations_path) {
        if !extend_schemas.iter().any(|s| s.table_name == table) {
            extend_schemas.push(ParsedSchema {
                table_name: table,
                ..Default::default()
            });
        }
    }

    let mut planned: Vec<(ParsedSchema, Changes)> = Vec::new();
    for ext_schema in extend_schemas {
        let snap_path = extend_snapshot_file_path(migrations_path, &ext_schema.table_name);

        let changes = if Path::new(&snap_path).exists() {
            let previous = previous_snapshot(&snap_path, &ext_schema)?;
            diff_schemas(&previous, &ext_schema)
        } else {
            // First time — all columns are new (ADD COLUMN). Never a new
            // table: the framework's already exists.
            Changes {
                table_name: ext_schema.table_name.clone(),
                added_columns: ext_schema.columns.clone(),
                ..Default::default()
            }
        };

        if changes.is_empty() {
            continue;
        }
        planned.push((ext_schema, changes));
    }

    Ok(planned)
}
