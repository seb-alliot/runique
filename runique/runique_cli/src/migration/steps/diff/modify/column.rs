//! A column whose definition changed.
use super::redefined::redefined;
use crate::migration::steps::diff::Columns;
use crate::migration::types::ParsedColumn;

/// (as it was, as it is), for every column that is in both schemas and
/// redefined.
pub(crate) fn column(
    prev_cols: &Columns<'_>,
    curr_cols: &Columns<'_>,
    renamed: &[(String, String)],
) -> Vec<(ParsedColumn, ParsedColumn)> {
    let mut modified: Vec<(ParsedColumn, ParsedColumn)> = curr_cols
        .iter()
        .filter_map(|(name, curr)| {
            let prev = prev_cols.get(name)?;
            redefined(prev, curr).then(|| ((*prev).clone(), (*curr).clone()))
        })
        .collect();
    // A renamed column is compared to what it was under its former name. It
    // joins the list under its new name: the rename is written first, so the
    // modification, and its undoing, find the column as it is called by then.
    for (old, new) in renamed {
        if let (Some(prev), Some(curr)) = (prev_cols.get(old.as_str()), curr_cols.get(new.as_str()))
            && redefined(prev, curr)
        {
            let was = ParsedColumn {
                name: new.clone(),
                ..(*prev).clone()
            };
            modified.push((was, (*curr).clone()));
        }
    }
    modified
}
