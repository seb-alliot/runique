//! Questions asked of a column.
use crate::migration::types::ParsedColumn;

/// Whether a column's definition changed, its name aside.
pub(crate) fn redefined(prev: &ParsedColumn, curr: &ParsedColumn) -> bool {
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

/// Whether going from `old` to `new` column length can drop data on some
/// engine. No length is unbounded on Postgres but `VARCHAR(255)` on MySQL, so
/// both directions out of "no length" are checked against each.
pub fn length_may_shrink(old: Option<u32>, new: Option<u32>) -> bool {
    match (old, new) {
        (Some(old), Some(new)) => new < old,
        (None, Some(_)) => true,
        (Some(old), None) => old > 255,
        (None, None) => false,
    }
}

/// Whether going from `old` to `new` decimal precision can drop data. No
/// precision is unbounded on Postgres and `DECIMAL(10, 0)` on MySQL, so a
/// first precision always may: fewer digits before the point, or after it.
pub fn decimal_may_shrink(old: Option<(u32, u32)>, new: Option<(u32, u32)>) -> bool {
    match (old, new) {
        (Some((old_p, old_s)), Some((new_p, new_s))) => {
            new_s < old_s || new_p.saturating_sub(new_s) < old_p.saturating_sub(old_s)
        }
        (None, Some(_)) => true,
        (_, None) => false,
    }
}
