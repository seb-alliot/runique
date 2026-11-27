//! A column dropped.
use super::*;

pub(crate) fn column(all_changes: &[Changes]) -> Vec<Destructive> {
    all_changes
        .iter()
        .flat_map(|c| {
            c.dropped_columns.iter().map(move |col| {
                found(
                    c,
                    DestructiveItem::DropColumn {
                        column: col.name.clone(),
                    },
                )
            })
        })
        .collect()
}
