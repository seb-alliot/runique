//! A foreign key dropped: rows may be left pointing nowhere.
use super::*;

pub(crate) fn key(all_changes: &[Changes]) -> Vec<Destructive> {
    all_changes
        .iter()
        .flat_map(|c| {
            c.dropped_fks.iter().map(move |fk| {
                found(
                    c,
                    DestructiveKind::DropKey {
                        column: fk.from_column.clone(),
                        to_table: fk.to_table.clone(),
                    },
                )
            })
        })
        .collect()
}
