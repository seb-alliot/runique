//! Questions asked of a schema.
use crate::migration::types::{ParsedColumn, ParsedSchema};

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

/// Columns that actually exist in the database (excluding ignored and PK)
///
/// # Example
///
/// ```rust, ignore
/// use runique_cli::migration::types::{ParsedSchema, ParsedColumn};
/// use runique_cli::migration::db_columns;
/// let schema = ParsedSchema {
///     table_name: "t".to_string(),
///     columns: vec![
///         ParsedColumn { name: "id".to_string(), col_type: "int".to_string(), nullable: false, unique: false, ignored: false },
///         ParsedColumn { name: "name".to_string(), col_type: "string".to_string(), nullable: false, unique: false, ignored: false },
///         ParsedColumn { name: "tmp".to_string(), col_type: "string".to_string(), nullable: false, unique: false, ignored: true },
///     ],
///     primary_key: Some(ParsedColumn { name: "id".to_string(), col_type: "int".to_string(), nullable: false, unique: false, ignored: false }),
///     foreign_keys: vec![],
///     indexes: vec![],
/// };
/// let cols = db_columns(&schema);
/// assert_eq!(cols.len(), 1);
/// assert_eq!(cols[0].name, "name");
/// ```
pub fn db_columns(schema: &ParsedSchema) -> Vec<&ParsedColumn> {
    let pk_name = schema.primary_key.as_ref().map(|pk| pk.name.as_str());
    schema
        .columns
        .iter()
        .filter(|c| !c.ignored && Some(c.name.as_str()) != pk_name)
        .collect()
}
