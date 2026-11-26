//! A column added.
use crate::migration::steps::diff::Columns;
use crate::migration::types::ParsedColumn;

/// The columns of the model the snapshot doesn't have, the renamed ones
/// aside.
pub(crate) fn column(
    prev_cols: &Columns<'_>,
    curr_cols: &Columns<'_>,
    renamed: &[(String, String)],
) -> Vec<ParsedColumn> {
    curr_cols
        .values()
        .filter(|c| !prev_cols.contains_key(c.name.as_str()))
        .filter(|c| !renamed.iter().any(|(_, new)| *new == c.name))
        .map(|c| (*c).clone())
        .collect()
}
