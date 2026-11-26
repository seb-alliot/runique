//! A value added to a string enum.
use crate::migration::steps::diff::{Columns, enum_columns};
use std::collections::HashSet;

/// (column, enum, value): the values the model has and the snapshot has not,
/// the new side of a rename aside.
pub(crate) fn enum_value(
    prev_cols: &Columns<'_>,
    curr_cols: &Columns<'_>,
    renamed: &[(String, String, String, String)],
) -> Vec<(String, String, String)> {
    let mut added = Vec::new();
    for (name, prev, curr) in enum_columns(prev_cols, curr_cols) {
        let prev_set: HashSet<&str> = prev.enum_string_values.iter().map(|s| s.as_str()).collect();
        let curr_set: HashSet<&str> = curr.enum_string_values.iter().map(|s| s.as_str()).collect();
        let enum_name = curr.enum_name.as_deref().unwrap_or(name).to_string();
        for value in curr_set.difference(&prev_set) {
            if renamed
                .iter()
                .any(|(column, _, _, new)| column == name && new == value)
            {
                continue;
            }
            added.push((name.to_string(), enum_name.clone(), value.to_string()));
        }
    }
    added
}
