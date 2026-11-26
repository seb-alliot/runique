use crate::migration::types::ParsedIndex;

/// The column of a foreign key's own index (`idx_{table}_{column}`, one column,
/// not unique) — what `to_schema` adds for every `belongs_to`.
pub(crate) fn fk_index_column<'a>(table: &str, idx: &'a ParsedIndex) -> Option<&'a str> {
    match idx.columns.as_slice() {
        [col] if !idx.unique && idx.name == format!("idx_{table}_{col}") => Some(col),
        _ => None,
    }
}
