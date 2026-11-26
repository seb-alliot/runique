use crate::migration::types::{ColumnType, ParsedColumn};

#[doc = include_str!("../../../../../doc-tests/migration/migration_col_type.md")]
/// The `ColumnDef` type method for a column, with its length, or its
/// precision and scale, when it has one.
pub fn col_type_method(column: &ParsedColumn) -> String {
    match (column.col_type, column.max_length, column.decimal_len) {
        (ColumnType::String, Some(n), _) => format!("string_len({n})"),
        (ColumnType::Binary, Some(n), _) => format!("binary_len({n})"),
        (ColumnType::VarBinary, n, _) => format!("var_binary({})", n.unwrap_or(255)),
        (ColumnType::Decimal, _, Some((p, s))) => format!("decimal_len({p}, {s})"),
        (col_type, _, _) => col_type.method().to_string(),
    }
}
