//! A column whose length narrows: longer values would be cut or refused.
use super::*;
use crate::migration::steps::changes::length_may_shrink;

pub(crate) fn column_length(all_changes: &[Changes]) -> Vec<Destructive> {
    all_changes
        .iter()
        .flat_map(|c| {
            c.modified_columns
                .iter()
                .filter(|(old, new)| {
                    old.col_type == new.col_type
                        && length_may_shrink(old.max_length, new.max_length)
                })
                .map(move |(old, new)| {
                    found(
                        c,
                        DestructiveKind::NarrowLength {
                            column: new.name.clone(),
                            from: old.max_length,
                            to: new.max_length,
                        },
                    )
                })
        })
        .collect()
}
