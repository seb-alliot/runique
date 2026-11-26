use crate::migration::types::ParsedSchema;

/// The database enums a model declares: the one its columns an its list
/// field use, each once.
pub(crate) fn enums_of(schema: &ParsedSchema, lists: &[ParsedSchema]) -> Vec<String> {
    let mut names: Vec<String> = std::iter::once(schema)
        .chain(lists)
        .flat_map(|table| &table.columns)
        .filter_map(|column| column.enum_name.clone())
        .collect();
    names.sort();
    names.dedup();
    names
}
