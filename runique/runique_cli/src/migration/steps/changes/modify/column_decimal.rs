//! A decimal whose precision narrows: values with more digits would be
//! rounded or refused.
use super::*;
use crate::migration::steps::changes::decimal_may_shrink;

pub(crate) fn column_decimal(all_changes: &[Changes]) -> Vec<Destructive> {
    all_changes
        .iter()
        .flat_map(|c| {
            c.modified_columns
                .iter()
                .filter(|(old, new)| {
                    old.col_type == new.col_type
                        && decimal_may_shrink(old.decimal_len, new.decimal_len)
                })
                .map(move |(old, new)| {
                    found(
                        c,
                        DestructiveItem::NarrowDecimal {
                            column: new.name.clone(),
                            from: old.decimal_len,
                            to: new.decimal_len,
                        },
                    )
                })
        })
        .collect()
}
