//! A `ON DELETE CASCADE` key added to a table that already has rows.
use super::*;
use crate::migration::types::{FkAction, TableChange};

// Adding a CASCADE constraint to existing data can trigger mass deletes if a parent is removed.
// Only a concern on existing tables: a table created in this same batch has no rows yet,
// so a CASCADE FK on a brand-new table carries no data-loss risk (avoids a false positive
// on first generation / from-scratch regeneration).
pub(crate) fn cascade_key(all_changes: &[Changes]) -> Vec<Destructive> {
    all_changes
        .iter()
        .filter(|c| c.table != TableChange::Created)
        .flat_map(|c| {
            c.added_fks
                .iter()
                .filter(|fk| fk.on_delete == FkAction::Cascade)
                .map(move |fk| {
                    found(
                        c,
                        DestructiveItem::AddCascadeKey {
                            column: fk.from_column.clone(),
                            to_table: fk.to_table.clone(),
                        },
                    )
                })
        })
        .collect()
}
