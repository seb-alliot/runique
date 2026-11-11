//! A column definition: its `ColumnDef` type method and its full rendering.
use crate::migration::types::ParsedColumn;

pub(super) fn render_column_def(col: &ParsedColumn) -> String {
    let null = if col.nullable {
        ".null()"
    } else {
        ".not_null()"
    };
    let uniq = if col.unique { ".unique_key()" } else { "" };
    let default = if col.has_default_now {
        ".default(Expr::current_timestamp())".to_string()
    } else if let Some(v) = &col.default_value {
        format!(".default({})", v)
    } else {
        String::new()
    };

    if !col.enum_string_values.is_empty() {
        format!(
            "ColumnDef::new_with_type(Alias::new(\"{name}\"), {ty}){null}{uniq}{default}",
            name = col.name,
            ty = enum_column_type(col),
            null = null,
            uniq = uniq,
            default = default,
        )
    } else {
        let ty = col_type_method(col);
        format!(
            "ColumnDef::new(Alias::new(\"{name}\")).{ty}{null}{uniq}{default}",
            name = col.name,
            ty = ty,
            null = null,
            uniq = uniq,
            default = default,
        )
    }
}

/// The type of an enum column, chosen when the migration runs: the Postgres
/// enum type there, a plain string elsewhere — the entity `model!{}`
/// generates reads it as a string off Postgres, and sea-query would render
/// `ColumnType::Enum` as a native `ENUM(...)` on MySQL/MariaDB, whose value
/// list the later migrations (value added, renamed) never update.
pub(super) fn enum_column_type(col: &ParsedColumn) -> String {
    let enum_name = col.enum_name.as_deref().unwrap_or(&col.name);
    let variants: Vec<String> = col
        .enum_string_values
        .iter()
        .map(|v| format!("Alias::new({v:?}).into_iden()"))
        .collect();
    format!(
        "if manager.get_connection().get_database_backend() == sea_orm::DbBackend::Postgres {{ ColumnType::Enum {{ name: Alias::new({enum_name:?}).into_iden(), variants: vec![{variants}] }} }} else {{ ColumnType::String(Default::default()) }}",
        variants = variants.join(", "),
    )
}

/// Returns the method name associated with a column type.
#[doc = include_str!("../../../doc-tests/migration/migration_col_type.md")]
/// The `ColumnDef` type method for a column, with its length, or its
/// precision and scale, when it has one.
pub fn col_type_method(column: &crate::migration::types::ParsedColumn) -> String {
    match (
        column.col_type.as_str(),
        column.max_length,
        column.decimal_len,
    ) {
        ("String", Some(n), _) => format!("string_len({n})"),
        ("Binary", Some(n), _) => format!("binary_len({n})"),
        ("VarBinary", n, _) => format!("var_binary({})", n.unwrap_or(255)),
        ("Decimal", _, Some((p, s))) => format!("decimal_len({p}, {s})"),
        (col_type, _, _) => col_type_to_method(col_type).to_string(),
    }
}

pub fn col_type_to_method(col_type: &str) -> &str {
    match col_type {
        "Text" => "text()",
        "TinyInteger" => "tiny_integer()",
        "SmallInteger" => "small_integer()",
        "Integer" => "integer()",
        "BigInteger" => "big_integer()",
        "Unsigned" => "unsigned()",
        "BigUnsigned" => "big_unsigned()",
        "Float" => "float()",
        "Double" => "double()",
        "Decimal" => "decimal()",
        "Boolean" => "boolean()",
        "DateTime" => "date_time()",
        "Timestamp" => "timestamp()",
        "TimestampWithTimeZone" => "timestamp_with_time_zone()",
        "Date" => "date()",
        "Time" => "time()",
        "Uuid" => "uuid()",
        "Json" => "json()",
        "JsonBinary" => "json_binary()",
        "Binary" => "binary()",
        "VarBinary" => "var_binary()",
        "Blob" => "blob()",
        "Char" => "char()",
        "Inet" => "inet()",
        "Cidr" => "cidr()",
        "MacAddr" => "mac_address()",
        "Interval" => "interval()",
        "Enum" => "enum_type()",
        _ => "string()",
    }
}
