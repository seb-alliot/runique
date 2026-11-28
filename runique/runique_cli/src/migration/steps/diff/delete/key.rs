//! A foreign key dropped.
use crate::migration::{
    steps::diff::key_identity,
    types::{ParsedFk, ParsedSchema},
};
use std::collections::HashSet;

pub(crate) fn key(previous: &ParsedSchema, current: &ParsedSchema) -> Vec<ParsedFk> {
    let now: HashSet<String> = current.foreign_keys.iter().map(key_identity).collect();
    previous
        .foreign_keys
        .iter()
        .filter(|fk| !now.contains(&key_identity(fk)))
        .cloned()
        .collect()
}
