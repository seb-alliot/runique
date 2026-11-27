use super::*;

/// The enums `schemas` use, each once, by name: its values, and every column
/// that uses it, in the order the tables and their columns are given in.
/// `schemas` are the tables of the models, of their list fields and of the
/// `extend!{}` blocks together: an enum is declared in one place, and may be
/// used by several of these tables.
pub fn enums_of_schemas(schemas: &[ParsedSchema]) -> Vec<ParsedEnum> {
    let mut enums: Vec<ParsedEnum> = Vec::new();
    for schema in schemas {
        for column in db_columns(schema) {
            if column.enum_string_values.is_empty() {
                continue;
            }
            // As the database names it, and as `with_enum_values` finds it.
            let name = column
                .enum_name
                .as_deref()
                .unwrap_or(&column.name)
                .to_lowercase();
            let place = EnumPlace {
                table: schema.table_name.clone(),
                column: column.name.clone(),
            };
            match enums.iter_mut().find(|known| known.name == name) {
                Some(known) => known.places.push(place),
                None => enums.push(ParsedEnum {
                    name,
                    values: column.enum_string_values.clone(),
                    places: vec![place],
                }),
            }
        }
    }
    enums.sort_by(|a, b| a.name.cmp(&b.name));
    enums
}
