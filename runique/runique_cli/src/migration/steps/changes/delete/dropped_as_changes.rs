use super::*;
use crate::migration::types::{ParsedSchema, TableChange};

/// The tables a run drops, as the guards read them: a dropped table is a
/// change like the others, and a destructive one.
pub fn dropped_as_changes(dropped: &[ParsedSchema]) -> Vec<Changes> {
    dropped
        .iter()
        .map(|schema| Changes {
            table_name: schema.table_name.clone(),
            table: TableChange::Deleted,
            ..Default::default()
        })
        .collect()
}
