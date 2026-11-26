//! An index dropped, told apart by name (see `create::index`).
use crate::migration::types::{ParsedIndex, ParsedSchema};
use std::collections::HashSet;

pub(crate) fn index(previous: &ParsedSchema, current: &ParsedSchema) -> Vec<ParsedIndex> {
    let now: HashSet<&str> = current.indexes.iter().map(|i| i.name.as_str()).collect();
    previous
        .indexes
        .iter()
        .filter(|i| !now.contains(i.name.as_str()))
        .cloned()
        .collect()
}
