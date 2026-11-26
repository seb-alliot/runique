//! The type changes `--force` let through and that are not migrated yet.
use crate::migration::types::Changes;

/// The columns whose type changed: `--force` let them through, but their
/// migration holds a warning comment only while the snapshot records the new
/// type, so the database keeps the old one. A change to or from an enum is
/// migrated (`build_enum_transition`) and isn't listed.
pub fn unmigrated_type_changes(changes: &[Changes]) -> Vec<String> {
    changes
        .iter()
        .flat_map(|c| {
            c.modified_columns
                .iter()
                .filter(|(old, new)| {
                    old.col_type != new.col_type
                        && old.enum_string_values.is_empty() == new.enum_string_values.is_empty()
                })
                .map(|(old, new)| {
                    format!(
                        "  {}.{}: {} -> {}",
                        c.table_name, old.name, old.col_type, new.col_type
                    )
                })
        })
        .collect()
}
