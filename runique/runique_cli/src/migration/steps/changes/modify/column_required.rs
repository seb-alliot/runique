//! A column made required: its `NULL` rows have nothing to take.
use super::*;

pub(crate) fn column_required(all_changes: &[Changes]) -> Vec<Destructive> {
    all_changes
        .iter()
        .flat_map(|c| {
            c.modified_columns
                .iter()
                .filter(|(old, new)| old.nullable && !new.nullable && old.col_type == new.col_type)
                .map(move |(_, new)| {
                    found(
                        c,
                        DestructiveKind::MakeRequired {
                            column: new.name.clone(),
                        },
                    )
                })
        })
        .collect()
}
