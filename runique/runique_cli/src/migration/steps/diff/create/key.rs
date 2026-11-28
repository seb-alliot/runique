//! A foreign key added.
use crate::migration::{
    steps::diff::key_identity,
    types::{ParsedFk, ParsedSchema},
};
use std::collections::HashSet;

pub(crate) fn key(previous: &ParsedSchema, current: &ParsedSchema) -> Vec<ParsedFk> {
    let before: HashSet<String> = previous.foreign_keys.iter().map(key_identity).collect();
    current
        .foreign_keys
        .iter()
        .filter(|fk| !before.contains(&key_identity(fk)))
        .cloned()
        .collect()
}
