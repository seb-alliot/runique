use crate::migration::types::ParsedColumn;

/// Whether a column's definition changed, its name aside.
pub(super) fn redefined(prev: &ParsedColumn, curr: &ParsedColumn) -> bool {
    prev.col_type != curr.col_type
        || prev.nullable != curr.nullable
        || prev.unique != curr.unique
        || prev.has_default_now != curr.has_default_now
        || prev.max_length != curr.max_length
        || prev.decimal_len != curr.decimal_len
        // Becoming (or stopping being) an enum: `col_type` alone doesn't
        // capture this (an enum column keeps col_type "String"). Two enums
        // with different values are handled by the enum diff, not here.
        || prev.enum_string_values.is_empty() != curr.enum_string_values.is_empty()
}
