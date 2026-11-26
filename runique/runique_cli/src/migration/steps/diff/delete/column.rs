//! A column dropped.
use crate::migration::steps::diff::Columns;
use crate::migration::types::{ParsedColumn, ParsedSchema};

/// The columns of the snapshot the model has no more, the renamed ones
/// aside, in the order of the snapshot.
pub(crate) fn column(
    previous: &ParsedSchema,
    curr_cols: &Columns<'_>,
    renamed: &[(String, String)],
) -> Vec<ParsedColumn> {
    previous
        .columns
        .iter()
        .filter(|c| !c.ignored)
        .filter(|c| !curr_cols.contains_key(c.name.as_str()))
        .filter(|c| !renamed.iter().any(|(old, _)| *old == c.name))
        .cloned()
        .collect()
}
