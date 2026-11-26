//! A value of a string enum renamed.
use crate::migration::steps::diff::{Columns, enum_columns};
use std::collections::HashSet;

/// (column, enum, old value, new value). A rename is ONE logical operation:
/// its values are left out of what is added and dropped, so the generator
/// can emit `ALTER TYPE … RENAME VALUE` (PG) instead of add + drop + UPDATE.
/// Found by position: same index, value changed, old value gone, new value
/// fresh.
pub(crate) fn enum_value(
    prev_cols: &Columns<'_>,
    curr_cols: &Columns<'_>,
) -> Vec<(String, String, String, String)> {
    let mut renamed = Vec::new();
    for (name, prev, curr) in enum_columns(prev_cols, curr_cols) {
        let prev_set: HashSet<&str> = prev.enum_string_values.iter().map(|s| s.as_str()).collect();
        let curr_set: HashSet<&str> = curr.enum_string_values.iter().map(|s| s.as_str()).collect();
        let enum_name = curr.enum_name.as_deref().unwrap_or(name).to_string();
        for (i, new_val) in curr.enum_string_values.iter().enumerate() {
            if let Some(old_val) = prev.enum_string_values.get(i)
                && old_val != new_val
                && prev_set.contains(old_val.as_str())
                && !curr_set.contains(old_val.as_str())
                && !prev_set.contains(new_val.as_str())
            {
                renamed.push((
                    name.to_string(),
                    enum_name.clone(),
                    old_val.clone(),
                    new_val.clone(),
                ));
            }
        }
    }
    renamed
}
