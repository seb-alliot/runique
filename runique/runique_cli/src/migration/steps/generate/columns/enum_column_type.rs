use crate::migration::types::ParsedColumn;

/// The type of an enum column, chosen when the migration runs: the Postgres
/// enum type there, a plain string elsewhere — the entity `model!{}`
/// generates reads it as a string off Postgres, and sea-query would render
/// `ColumnType::Enum` as a native `ENUM(...)` on MySQL, whose value
/// list the later migrations (value added, renamed) never update.
pub(crate) fn enum_column_type(col: &ParsedColumn) -> String {
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
