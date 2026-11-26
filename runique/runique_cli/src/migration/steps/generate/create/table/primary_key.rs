use crate::migration::types::{ColumnType, ParsedColumn};

pub(super) fn primary_key(pk: &ParsedColumn) -> String {
    let ty = pk.col_type.method();

    // auto_increment only makes sense for integer-ish PK.
    let autoinc_ok = matches!(
        pk.col_type,
        ColumnType::Integer
            | ColumnType::BigInteger
            | ColumnType::SmallInteger
            | ColumnType::TinyInteger
            | ColumnType::Unsigned
            | ColumnType::BigUnsigned
    );

    let mut s = format!(
        "                    .col(ColumnDef::new(Alias::new(\"{name}\")).{ty}.not_null()",
        name = pk.name,
        ty = ty
    );

    if autoinc_ok {
        s.push_str(".auto_increment()");
    }

    s.push_str(".primary_key())");
    s
}
