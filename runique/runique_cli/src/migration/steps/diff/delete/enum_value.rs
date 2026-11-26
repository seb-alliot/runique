//! A value taken out of a string enum.
use crate::migration::steps::diff::{Columns, enum_columns};
use std::collections::HashSet;

/// (column, enum, value): the values the snapshot has and the model has no
/// more, the old side of a rename aside. The enum is named as the snapshot
/// names it.
pub(crate) fn enum_value(
    prev_cols: &Columns<'_>,
    curr_cols: &Columns<'_>,
    renamed: &[(String, String, String, String)],
) -> Vec<(String, String, String)> {
    let mut dropped = Vec::new();
    for (name, prev, curr) in enum_columns(prev_cols, curr_cols) {
        let prev_set: HashSet<&str> = prev.enum_string_values.iter().map(|s| s.as_str()).collect();
        let curr_set: HashSet<&str> = curr.enum_string_values.iter().map(|s| s.as_str()).collect();
        let enum_name = prev.enum_name.as_deref().unwrap_or(name).to_string();
        for value in prev_set.difference(&curr_set) {
            if renamed
                .iter()
                .any(|(column, _, old, _)| column == name && old == value)
            {
                continue;
            }
            dropped.push((name.to_string(), enum_name.clone(), value.to_string()));
        }
    }
    dropped
}
