use super::*;

/// One declared field as a migration column. Shared by `model!{}` and
/// `extend!{}`, which declare fields with the same grammar.
pub(crate) fn decl_to_column(decl: &FormFieldDecl, enums: &[EnumDef]) -> ParsedColumn {
    let field = form_field_to_field_def(decl);
    let has = |wanted: fn(&FieldOption) -> bool| field.options.iter().any(wanted);
    let auto_now = has(|o| matches!(o, FieldOption::AutoNow));
    let auto_now_update = has(|o| matches!(o, FieldOption::AutoNowUpdate));
    let name = field.name.to_string();

    let ty = field.column_type();
    let max_length = match &ty {
        FieldType::Varchar(n) | FieldType::VarBinary(n) => Some(*n),
        FieldType::Binary(n) => field.kind.byte_limit(*n),
        _ => None,
    };
    let decimal_len = match &ty {
        FieldType::Decimal(len) => *len,
        _ => None,
    };
    // Same rule as `ColumnDef::auto_now`: a date-time column, except that a
    // `timestamp_tz` keeps its time zone (its entity field is a `DateTime<Utc>`).
    let col_type = if (auto_now || auto_now_update) && !matches!(ty, FieldType::TimestampTz) {
        ColumnType::DateTime
    } else {
        col_type(&ty, enums)
    };

    // Only string-backed enums carry values (the generator emits CREATE TYPE for them on PG).
    let (enum_name, enum_string_values) = enum_values(&ty, enums);

    let default_value = field.options.iter().find_map(|o| match o {
        FieldOption::Default(lit) => Some(lit.to_token_stream().to_string()),
        _ => None,
    });
    let renamed_from = decl.attrs.iter().find_map(|a| match a {
        FormFieldAttr::RenamedFrom(old) => Some(old.clone()),
        _ => None,
    });

    ParsedColumn {
        col_type,
        nullable: has(|o| matches!(o, FieldOption::Nullable)),
        unique: has(|o| matches!(o, FieldOption::Unique)),
        ignored: has(|o| matches!(o, FieldOption::Readonly)),
        has_default_now: auto_now || auto_now_update,
        default_value,
        enum_name,
        enum_string_values,
        renamed_from,
        max_length,
        decimal_len,
        name,
    }
}
