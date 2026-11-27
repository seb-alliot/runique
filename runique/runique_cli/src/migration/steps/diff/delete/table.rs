//! A table dropped.
use crate::migration::*;

/// Every table of the framework starts with this.
const FRAMEWORK_TABLE_PREFIX: &str = "eihwaz_";

/// The tables to drop: the ones Runique migrated (`snapshots`, the snapshots
/// of `snapshots/` itself) that no scanned model produces any more, by name.
/// A table of the framework is never dropped, nor one of `kept`: the paused
/// tables, and the former name of a table this run renames.
pub fn dropped_tables(
    snapshots: Vec<ParsedSchema>,
    schemas: &[ParsedSchema],
    kept: &[String],
) -> Vec<ParsedSchema> {
    let mut dropped: Vec<ParsedSchema> = snapshots
        .into_iter()
        .filter(|snapshot| {
            !snapshot.table_name.starts_with(FRAMEWORK_TABLE_PREFIX)
                && !kept.contains(&snapshot.table_name)
                && !schemas.iter().any(|s| s.table_name == snapshot.table_name)
        })
        .collect();
    dropped.sort_by(|a, b| a.table_name.cmp(&b.table_name));
    dropped
}
