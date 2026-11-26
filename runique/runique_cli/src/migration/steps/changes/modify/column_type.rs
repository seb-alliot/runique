//! A column whose type changes, or that enters or leaves an enum.
use super::*;

pub(crate) fn column_type(all_changes: &[Changes]) -> Vec<Destructive> {
    all_changes
        .iter()
        .flat_map(|c| {
            c.modified_columns
                .iter()
                // `col_type` alone misses becoming (or stopping being) an enum — an enum
                // column keeps col_type "String", the shape change lives in
                // `enum_string_values` instead. Without this, a string->enum column would
                // generate silently (no `--force` prompt) with only a code comment warning
                // the developer might never see.
                .filter(|(old, new)| {
                    old.col_type != new.col_type
                        || old.enum_string_values.is_empty() != new.enum_string_values.is_empty()
                })
                .map(move |(old, new)| {
                    let column = old.name.clone();
                    let kind = if old.col_type != new.col_type {
                        DestructiveKind::ChangeType {
                            column,
                            from: old.col_type.to_string(),
                            to: new.col_type.to_string(),
                        }
                    } else if new.enum_string_values.is_empty() {
                        DestructiveKind::LeaveEnum {
                            column,
                            to: new.col_type.to_string(),
                        }
                    } else {
                        DestructiveKind::EnterEnum {
                            column,
                            from: old.col_type.to_string(),
                        }
                    };
                    found(c, kind)
                })
        })
        .collect()
}
