//! Length limit of the generated names (FK constraints, indexes) per engine.
use crate::migration::types::Changes;

/// Scans a set of changes for FK constraint / index names that would exceed
/// MariaDB/MySQL's 64-character identifier limit (63 used here as the shared safe bound —
/// Postgres's own NAMEDATALEN-1 limit is one byte tighter). Checked once, upfront, over
/// the whole plan — before any migration file is generated — so the string-building
/// generators (`migration/generate/`) and the DSL-to-schema conversion (`to_schema.rs`) stay
/// pure and infallible. There is no `--force` escape hatch here, unlike
/// `check_destructive`: Runique won't guess a shortened name for you (that would make it
/// unpredictable from the `model!{}` declaration alone), so a human has to rename
/// something regardless.
pub fn collect_long_identifier_messages(all_changes: &[Changes]) -> Vec<String> {
    const MAX_LEN: usize = 63;
    let mut messages = Vec::new();

    for change in all_changes {
        for fk in change.added_fks.iter().chain(change.dropped_fks.iter()) {
            let name = format!(
                "{}_{}_{}_fkey",
                change.table_name, fk.from_column, fk.to_table
            );
            if name.len() > MAX_LEN {
                messages.push(format!(
                    "  {name} ({len} characters, max {MAX_LEN}) — shorten the table name, '{col}', or the target table name",
                    len = name.len(),
                    col = fk.from_column,
                ));
            }
        }
        for idx in change
            .added_indexes
            .iter()
            .chain(change.dropped_indexes.iter())
        {
            if idx.name.len() > MAX_LEN {
                messages.push(format!(
                    "  {} ({} characters, max {MAX_LEN}) — shorten the table or column names in this index",
                    idx.name,
                    idx.name.len(),
                ));
            }
        }
    }
    messages
}
