use super::{col_type_method, enum_column_type};
use crate::migration::types::ParsedColumn;

pub(crate) fn render_column_def(col: &ParsedColumn) -> String {
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
