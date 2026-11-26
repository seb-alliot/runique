//! A table dropped: every row of it is lost.
use super::*;
use crate::migration::types::TableChange;

pub(crate) fn table(all_changes: &[Changes]) -> Vec<Destructive> {
    all_changes
        .iter()
        .filter(|c| c.table == TableChange::Deleted)
        .map(|c| found(c, DestructiveKind::DropTable))
        .collect()
}
