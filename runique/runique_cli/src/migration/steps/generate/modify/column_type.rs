//! A column whose type changes: not migrated yet, the file says so.
use crate::migration::types::ColumnType;

pub(crate) fn column_type(column: &str, from: ColumnType, to: ColumnType) -> String {
    format!(
        "        // WARNING: type change on column '{col}': {old} -> {new}\n        // Manual migration required.\n\n",
        col = column,
        old = from,
        new = to
    )
}
