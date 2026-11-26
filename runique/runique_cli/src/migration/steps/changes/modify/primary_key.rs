//! The primary key of a table against the one its snapshot recorded.
use crate::migration::types::ParsedSchema;

/// How the primary key of `current` differs from the one its snapshot
/// (`previous`) recorded: its name, its type, or both. `None` when the are
/// the same, or when a side has no key to compare.
pub fn primary_key_change(previous: &ParsedSchema, current: &ParsedSchema) -> Option<String> {
    let old = previous.primary_key.as_ref()?;
    let new = current.primary_key.as_ref()?;
    if old.name == new.name && old.col_type == new.col_type {
        return None;
    }
    Some(format!(
        "  {}: {} ({}) -> {} ({})",
        current.table_name, old.name, old.col_type, new.name, new.col_type
    ))
}
