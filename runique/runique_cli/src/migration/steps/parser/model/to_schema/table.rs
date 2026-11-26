use super::*;

pub(crate) fn model_to_parsed_schema(model: &ModelInput) -> ParsedSchema {
    let primary_key = Some(ParsedColumn {
        name: model.pk.name.to_string(),
        col_type: ColumnType::of_pk(model.pk.ty),
        ..ParsedColumn::default()
    });

    let columns: Vec<ParsedColumn> = model
        .form_fields
        .iter()
        .map(|decl| decl_to_column(decl, &model.enums))
        .collect();

    let foreign_keys: Vec<ParsedFk> = model
        .relations
        .iter()
        .filter_map(|rel| match rel {
            RelationDef::BelongsTo {
                model: target,
                via,
                on_delete,
                on_update,
            } => Some(ParsedFk {
                from_column: via.to_string(),
                // The target's module name; `scan_entities` swaps in its real
                // table and primary key.
                to_table: snake_case(&target.to_string()),
                to_column: "id".to_string(),
                on_delete: *on_delete,
                on_update: *on_update,
            }),
            _ => None,
        })
        .collect();

    let table = model.table.clone();
    let names =
        |cols: &[syn::Ident]| -> Vec<String> { cols.iter().map(|c| c.to_string()).collect() };
    let mut indexes = Vec::new();
    if let Some(meta) = &model.meta {
        for cols in &meta.unique_together {
            let columns = names(cols);
            indexes.push(ParsedIndex {
                name: format!("{}_{}_uniq", table, columns.join("_")),
                columns,
                unique: true,
            });
        }
        for cols in &meta.indexes {
            let columns = names(cols);
            indexes.push(ParsedIndex {
                name: format!("idx_{}_{}", table, columns.join("_")),
                columns,
                unique: false,
            });
        }
    }

    // One index per foreign key column, as Django does: joins and ON DELETE
    // read it, and Postgres/SQLite never create one by themselves (MySQL
    // drops its implicit one once this exists). Skipped when a unique column
    // or a declared index already starts with it.
    for fk in &foreign_keys {
        let covered = columns.iter().any(|c| c.name == fk.from_column && c.unique)
            || indexes
                .iter()
                .any(|i| i.columns.first() == Some(&fk.from_column));
        if !covered {
            indexes.push(ParsedIndex {
                name: format!("idx_{}_{}", table, fk.from_column),
                columns: vec![fk.from_column.clone()],
                unique: false,
            });
        }
    }

    ParsedSchema {
        table_name: table,
        primary_key,
        columns,
        foreign_keys,
        indexes,
    }
}
