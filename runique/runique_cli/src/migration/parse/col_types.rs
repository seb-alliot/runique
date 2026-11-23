//! Recognizing a column type from the method names of a column definition
//! (the reverse of `col_type_method`, used when reading existing code).

/// Infers the semantic column type name (e.g. `"Integer"`, `"Text"`, `"Uuid"`) from
/// the set of builder method names called in a `ColumnDef` chain (as parsed from the
/// `model!{}` DSL builder syntax). Falls back to `"String"` when nothing matches.
pub fn detect_col_type_builder(methods: &[String]) -> String {
    // Binaries
    if methods.contains(&"blob".to_string()) {
        "Blob".to_string()
    } else if methods.contains(&"binary".to_string()) || methods.contains(&"binary_len".to_string())
    {
        "Binary".to_string()
    } else if methods.contains(&"var_binary".to_string()) {
        "VarBinary".to_string()
    }
    // Texts
    else if methods.contains(&"text".to_string()) {
        "Text".to_string()
    } else if methods.contains(&"char".to_string()) || methods.contains(&"char_len".to_string()) {
        "Char".to_string()
    } else if methods.contains(&"varchar".to_string())
        || methods.contains(&"string_len".to_string())
    {
        "String".to_string()
    }
    // Integers
    else if methods.contains(&"tiny_integer".to_string()) {
        "TinyInteger".to_string()
    } else if methods.contains(&"small_integer".to_string()) {
        "SmallInteger".to_string()
    } else if methods.contains(&"big_unsigned".to_string()) {
        "BigUnsigned".to_string()
    } else if methods.contains(&"unsigned".to_string()) {
        "Unsigned".to_string()
    } else if methods.contains(&"big_integer".to_string()) {
        "BigInteger".to_string()
    } else if methods.contains(&"integer".to_string()) {
        "Integer".to_string()
    }
    // Numerics
    else if methods.contains(&"float".to_string()) {
        "Float".to_string()
    } else if methods.contains(&"double".to_string()) {
        "Double".to_string()
    } else if methods.contains(&"decimal".to_string())
        || methods.contains(&"decimal_len".to_string())
    {
        "Decimal".to_string()
    }
    // Boolean
    else if methods.contains(&"boolean".to_string()) {
        "Boolean".to_string()
    }
    // Date/Time
    else if methods.contains(&"timestamp_tz".to_string()) {
        "TimestampWithTimeZone".to_string()
    } else if methods.contains(&"timestamp".to_string()) {
        "Timestamp".to_string()
    } else if methods.contains(&"datetime".to_string())
        || methods.contains(&"auto_now".to_string())
        || methods.contains(&"auto_now_update".to_string())
    {
        "DateTime".to_string()
    } else if methods.contains(&"date".to_string()) {
        "Date".to_string()
    } else if methods.contains(&"time".to_string()) {
        "Time".to_string()
    }
    // UUID
    else if methods.contains(&"uuid".to_string()) {
        "Uuid".to_string()
    }
    // JSON
    else if methods.contains(&"json_binary".to_string()) {
        "JsonBinary".to_string()
    } else if methods.contains(&"json".to_string()) {
        "Json".to_string()
    }
    // Fallback
    else {
        "String".to_string()
    }
}

/// Same as [`detect_col_type_builder`], but for method chains found in a generated
/// SeaORM snapshot — recognizes a few additional SeaORM-specific spellings (e.g.
/// `date_time`, `timestamp_with_time_zone`) and PostgreSQL/enum-specific methods.
pub fn detect_col_type_seaorm(methods: &[String]) -> String {
    // Binaries
    if methods.contains(&"blob".to_string()) {
        "Blob".to_string()
    } else if methods.contains(&"binary".to_string()) || methods.contains(&"binary_len".to_string())
    {
        "Binary".to_string()
    } else if methods.contains(&"var_binary".to_string()) {
        "VarBinary".to_string()
    }
    // Texts
    else if methods.contains(&"text".to_string()) {
        "Text".to_string()
    } else if methods.contains(&"char".to_string()) || methods.contains(&"char_len".to_string()) {
        "Char".to_string()
    }
    // Integers (order: most specific to most general)
    else if methods.contains(&"tiny_integer".to_string()) {
        "TinyInteger".to_string()
    } else if methods.contains(&"small_integer".to_string()) {
        "SmallInteger".to_string()
    } else if methods.contains(&"big_unsigned".to_string()) {
        "BigUnsigned".to_string()
    } else if methods.contains(&"unsigned".to_string()) {
        "Unsigned".to_string()
    } else if methods.contains(&"big_integer".to_string()) {
        "BigInteger".to_string()
    } else if methods.contains(&"integer".to_string()) {
        "Integer".to_string()
    }
    // Numerics
    else if methods.contains(&"float".to_string()) {
        "Float".to_string()
    } else if methods.contains(&"double".to_string()) {
        "Double".to_string()
    } else if methods.contains(&"decimal".to_string())
        || methods.contains(&"decimal_len".to_string())
    {
        "Decimal".to_string()
    }
    // Boolean
    else if methods.contains(&"boolean".to_string()) {
        "Boolean".to_string()
    }
    // Date/Time
    else if methods.contains(&"timestamp_tz".to_string())
        || methods.contains(&"timestamp_with_time_zone".to_string())
    {
        "TimestampWithTimeZone".to_string()
    } else if methods.contains(&"timestamp".to_string()) {
        "Timestamp".to_string()
    } else if methods.contains(&"date_time".to_string())
        || methods.contains(&"auto_now".to_string())
        || methods.contains(&"auto_now_update".to_string())
    {
        "DateTime".to_string()
    } else if methods.contains(&"date".to_string()) {
        "Date".to_string()
    } else if methods.contains(&"time".to_string()) {
        "Time".to_string()
    }
    // UUID
    else if methods.contains(&"uuid".to_string()) {
        "Uuid".to_string()
    }
    // JSON
    else if methods.contains(&"json_binary".to_string()) {
        "JsonBinary".to_string()
    } else if methods.contains(&"json".to_string()) {
        "Json".to_string()
    }
    // PostgreSQL specific
    else if methods.contains(&"inet".to_string()) {
        "Inet".to_string()
    } else if methods.contains(&"cidr".to_string()) {
        "Cidr".to_string()
    } else if methods.contains(&"mac_address".to_string()) {
        "MacAddr".to_string()
    } else if methods.contains(&"interval".to_string()) {
        "Interval".to_string()
    }
    // Enum
    else if methods.contains(&"enum_type".to_string())
        || methods.contains(&"enumeration".to_string())
    {
        "Enum".to_string()
    }
    // Fallback
    else {
        "String".to_string()
    }
}
