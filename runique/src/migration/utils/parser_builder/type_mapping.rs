//! DSL type name → SeaORM column type mapping — the single table shared by
//! both the `pk:` position and ordinary fields.
use crate::utils::trad::tf;

/// Converts a DSL type (v1 SQL or v2 semantic) to a SeaORM type name.
/// `default` covers a type name this table doesn't recognize; the two call
/// sites below need different ones (a `pk:` position with no clear type still
/// wants an auto-increment `Integer`, an ordinary unknown field is safest as
/// `String`), but every OTHER type name must resolve identically everywhere —
/// two separate match arms for the same table is exactly how a `Pk`-typed FK
/// field ended up correctly resolved in the pk-position table but silently
/// misclassified as `String` in the field one (fixed 2026-09-01, then
/// collapsed here so the two can no longer drift apart again).
fn dsl_type_to_col_type(ty: &str, default: &str) -> String {
    match ty {
        // v1 SQL
        "String" | "char" | "varchar" => "String".to_string(),
        // "text" in v2 = short text field (VARCHAR) — same behavior as String in v1
        "text" => "String".to_string(),
        "i8" => "TinyInteger".to_string(),
        "i16" => "SmallInteger".to_string(),
        "i32" | "integer" => "Integer".to_string(),
        "i64" | "big_integer" | "bigint" => "BigInteger".to_string(),
        "u32" => "Unsigned".to_string(),
        "u64" => "BigUnsigned".to_string(),
        "f32" => "Float".to_string(),
        "f64" | "float" | "percent" => "Double".to_string(),
        "decimal" => "Decimal".to_string(),
        "bool" => "Boolean".to_string(),
        "date" => "Date".to_string(),
        "time" => "Time".to_string(),
        "datetime" | "timestamp" => "DateTime".to_string(),
        "timestamp_tz" => "TimestampWithTimeZone".to_string(),
        "uuid" => "Uuid".to_string(),
        "json" | "json_binary" => "Json".to_string(),
        "binary" | "blob" => "Binary".to_string(),
        "var_binary" => "VarBinary".to_string(),
        "inet" | "cidr" | "mac_address" | "interval" | "ip" => "String".to_string(),
        // v2 semantic text → String (VARCHAR) or Text
        "email" | "url" | "password" | "slug" | "color" | "phone" => "String".to_string(),
        "richtext" | "textarea" => "Text".to_string(),
        // v2 files → String (JSON path)
        "image" | "document" | "file" => "String".to_string(),
        // v2 choice/radio → resolved via enum_ref in dsl_to_parsed_schema, fallback String
        "choice" | "radio" => "String".to_string(),
        // explicit int
        "int" => "Integer".to_string(),
        // `Pk` usable on any field, not just `pk:` (e.g. an FK typed `Pk` to stay in
        // sync with the referenced table's PK type under big-pk/pk-uuid).
        "Pk" => pk_alias_col_type(),
        _ => {
            eprintln!("{}", tf("makemigrations.unknown_dsl_type", &[ty, default]));
            default.to_string()
        }
    }
}

pub(super) fn dsl_field_type_to_col_type(ty: &str) -> String {
    dsl_type_to_col_type(ty, "String")
}

pub(super) fn dsl_pk_to_col_type(ty: &str) -> String {
    dsl_type_to_col_type(ty, "Integer")
}

/// Column type for the generic `Pk` DSL keyword — follows the same global alias as
/// `runique::utils::config::Pk`, so the migration SQL matches the ORM's actual column type.
#[cfg(feature = "pk-uuid")]
fn pk_alias_col_type() -> String {
    "Uuid".to_string()
}
#[cfg(all(feature = "big-pk", not(feature = "pk-uuid")))]
fn pk_alias_col_type() -> String {
    "BigInteger".to_string()
}
#[cfg(not(any(feature = "big-pk", feature = "pk-uuid")))]
fn pk_alias_col_type() -> String {
    "Integer".to_string()
}
