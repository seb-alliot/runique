use super::*;

/// The table of each list field (`checkbox` / `multichoice`): one row per
/// value chosen, `(owner_id, value)` unique, deleted with its owner. Named
/// `{owner table}_{field}`: owner and field together, so two models, or two
/// fields of one model, using the same enum never share it.
pub(crate) fn list_tables(model: &ModelInput, owner: &ParsedSchema) -> Vec<ParsedSchema> {
    let Some(owner_pk) = owner.primary_key.as_ref() else {
        return Vec::new();
    };
    model
        .lists
        .iter()
        .map(|decl| {
            let table = format!("{}_{}", owner.table_name, decl.name);
            let enum_ref = decl.attrs.iter().find_map(|a| match a {
                FormFieldAttr::EnumRef(id) => Some(id.clone()),
                _ => None,
            });
            let ty = enum_ref.map_or(FieldType::String, FieldType::Enum);
            let (enum_name, enum_string_values) = enum_values(&ty, &model.enums);
            let column = |name: &str, col_type: ColumnType| ParsedColumn {
                name: name.to_string(),
                col_type,
                ..ParsedColumn::default()
            };
            ParsedSchema {
                primary_key: Some(ParsedColumn {
                    name: "id".to_string(),
                    col_type: ColumnType::BigInteger,
                    ..ParsedColumn::default()
                }),
                columns: vec![
                    column("owner_id", owner_pk.col_type),
                    ParsedColumn {
                        enum_name,
                        enum_string_values,
                        ..column("value", col_type(&ty, &model.enums))
                    },
                ],
                foreign_keys: vec![ParsedFk {
                    from_column: "owner_id".to_string(),
                    to_table: owner.table_name.clone(),
                    to_column: owner_pk.name.clone(),
                    on_delete: FkAction::Cascade,
                    on_update: FkAction::NoAction,
                }],
                indexes: vec![
                    ParsedIndex {
                        name: format!("{table}_owner_id_value_uniq"),
                        columns: vec!["owner_id".to_string(), "value".to_string()],
                        unique: true,
                    },
                    ParsedIndex {
                        name: format!("idx_{table}_value_owner_id"),
                        columns: vec!["value".to_string(), "owner_id".to_string()],
                        unique: false,
                    },
                ],
                table_name: table,
            }
        })
        .collect()
}
