//! An index added. Indexes are told apart by name: the name of an index
//! holds its columns and whether it is unique, so any change gives it another.
use crate::migration::types::{ParsedIndex, ParsedSchema};
use std::collections::HashSet;

pub(crate) fn index(previous: &ParsedSchema, current: &ParsedSchema) -> Vec<ParsedIndex> {
    let before: HashSet<&str> = previous.indexes.iter().map(|i| i.name.as_str()).collect();
    current
        .indexes
        .iter()
        .filter(|i| !before.contains(i.name.as_str()))
        .cloned()
        .collect()
}
