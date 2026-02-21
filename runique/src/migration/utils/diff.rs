use std::collections::{HashMap, HashSet};

use crate::migration::utils::types::{Changes, ParsedColumn, ParsedSchema};

/// Colonnes qui existent réellement en base (hors ignored et hors PK)
///
/// # Exemple
///
/// ```rust
/// use runique::migration::utils::types::{ParsedSchema, ParsedColumn};
/// use runique::migration::utils::diff::db_columns;
/// let schema = ParsedSchema {
///     table_name: "t".to_string(),
///     columns: vec![
///         ParsedColumn { name: "id".to_string(), col_type: "int".to_string(), nullable: false, unique: false, ignored: false },
///         ParsedColumn { name: "name".to_string(), col_type: "string".to_string(), nullable: false, unique: false, ignored: false },
///         ParsedColumn { name: "tmp".to_string(), col_type: "string".to_string(), nullable: false, unique: false, ignored: true },
///     ],
///     primary_key: Some(ParsedColumn { name: "id".to_string(), col_type: "int".to_string(), nullable: false, unique: false, ignored: false }),
///     foreign_keys: vec![],
///     indexes: vec![],
/// };
/// let cols = db_columns(&schema);
/// assert_eq!(cols.len(), 1);
/// assert_eq!(cols[0].name, "name");
/// ```
pub fn db_columns(schema: &ParsedSchema) -> Vec<&ParsedColumn> {
    let pk_name = schema.primary_key.as_ref().map(|pk| pk.name.as_str());
    schema
        .columns
        .iter()
        .filter(|c| !c.ignored && Some(c.name.as_str()) != pk_name)
        .collect()
}

/// Calcule les différences entre deux schémas de table.
///
/// # Exemple
///
/// ```rust,ignore
/// // let changes = diff_schemas(&old_schema, &new_schema);
/// // assert!(changes.added_columns.len() > 0 || changes.dropped_columns.len() > 0);
/// ```
pub fn diff_schemas(previous: &ParsedSchema, current: &ParsedSchema) -> Changes {
    let prev_cols: HashMap<&str, &ParsedColumn> = db_columns(previous)
        .into_iter()
        .map(|c| (c.name.as_str(), c))
        .collect();
    let curr_cols: HashMap<&str, &ParsedColumn> = db_columns(current)
        .into_iter()
        .map(|c| (c.name.as_str(), c))
        .collect();

    let added_columns = curr_cols
        .values()
        .filter(|c| !prev_cols.contains_key(c.name.as_str()))
        .map(|c| (*c).clone())
        .collect();

    let dropped_columns: Vec<ParsedColumn> = previous
        .columns
        .iter()
        .filter(|c| !c.ignored)
        .filter(|c| !curr_cols.contains_key(c.name.as_str()))
        .cloned()
        .collect();

    let modified_columns = curr_cols
        .iter()
        .filter_map(|(name, curr)| {
            prev_cols.get(name).and_then(|prev| {
                if prev.col_type != curr.col_type
                    || prev.nullable != curr.nullable
                    || prev.unique != curr.unique
                {
                    Some(((*prev).clone(), (*curr).clone()))
                } else {
                    None
                }
            })
        })
        .collect();

    // FK
    let prev_fks: HashSet<String> = previous
        .foreign_keys
        .iter()
        .map(|fk| format!("{}->{}:{}", fk.from_column, fk.to_table, fk.to_column))
        .collect();
    let curr_fks: HashSet<String> = current
        .foreign_keys
        .iter()
        .map(|fk| format!("{}->{}:{}", fk.from_column, fk.to_table, fk.to_column))
        .collect();

    let added_fks = current
        .foreign_keys
        .iter()
        .filter(|fk| {
            !prev_fks.contains(&format!(
                "{}->{}:{}",
                fk.from_column, fk.to_table, fk.to_column
            ))
        })
        .cloned()
        .collect();

    let dropped_fks = previous
        .foreign_keys
        .iter()
        .filter(|fk| {
            !curr_fks.contains(&format!(
                "{}->{}:{}",
                fk.from_column, fk.to_table, fk.to_column
            ))
        })
        .cloned()
        .collect();

    // Indexes
    let prev_idx: HashSet<&str> = previous.indexes.iter().map(|i| i.name.as_str()).collect();
    let curr_idx: HashSet<&str> = current.indexes.iter().map(|i| i.name.as_str()).collect();

    let added_indexes = current
        .indexes
        .iter()
        .filter(|i| !prev_idx.contains(i.name.as_str()))
        .cloned()
        .collect();
    let dropped_indexes = previous
        .indexes
        .iter()
        .filter(|i| !curr_idx.contains(i.name.as_str()))
        .cloned()
        .collect();

    Changes {
        table_name: current.table_name.clone(),
        added_columns,
        dropped_columns,
        modified_columns,
        added_fks,
        dropped_fks,
        added_indexes,
        dropped_indexes,
        is_new_table: false,
    }
}
