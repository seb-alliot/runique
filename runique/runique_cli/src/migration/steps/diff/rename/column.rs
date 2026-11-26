//! A column renamed, through the explicit `renamed_from` hint.
use crate::migration::steps::diff::Columns;

/// (old name, new name). A rename is ONE operation: the old and the new
/// names are left out of what is dropped and added, so the migration writes
/// a RENAME COLUMN (no data loss). The old name must exist in the snapshot
/// and be gone from the model, and the new name must be genuinely new —
/// otherwise the hint is stale and the column falls back to add and drop.
pub(crate) fn column(prev_cols: &Columns<'_>, curr_cols: &Columns<'_>) -> Vec<(String, String)> {
    let mut renamed: Vec<(String, String)> = Vec::new();
    for (new_name, curr) in curr_cols {
        if let Some(old) = curr.renamed_from.as_deref()
            && prev_cols.contains_key(old)
            && !curr_cols.contains_key(old)
            && !prev_cols.contains_key(*new_name)
        {
            renamed.push((old.to_string(), new_name.to_string()));
        }
    }
    renamed
}
