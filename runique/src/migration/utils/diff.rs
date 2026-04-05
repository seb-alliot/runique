//! Calcul de diff entre deux [`ParsedSchema`] — colonnes ajoutées/supprimées/modifiées, FK, index, enum renames.
use std::collections::{HashMap, HashSet};

use crate::migration::utils::types::{Changes, ParsedColumn, ParsedFk, ParsedSchema};

/// Colonnes qui existent réellement en base (hors ignored et hors PK)
///
/// # Exemple
///
/// ```rust, ignore
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
                    || prev.has_default_now != curr.has_default_now
                    || prev.updated_at != curr.updated_at
                    || prev.created_at != curr.created_at
                {
                    Some(((*prev).clone(), (*curr).clone()))
                } else {
                    None
                }
            })
        })
        .collect();

    // FK — la clé inclut on_delete et on_update pour détecter les changements d'action
    let fk_key = |fk: &ParsedFk| {
        format!(
            "{}->{}:{}:{}:{}",
            fk.from_column, fk.to_table, fk.to_column, fk.on_delete, fk.on_update
        )
    };

    let prev_fks: HashSet<String> = previous.foreign_keys.iter().map(&fk_key).collect();
    let curr_fks: HashSet<String> = current.foreign_keys.iter().map(&fk_key).collect();

    let added_fks = current
        .foreign_keys
        .iter()
        .filter(|fk| !prev_fks.contains(&fk_key(fk)))
        .cloned()
        .collect();

    let dropped_fks = previous
        .foreign_keys
        .iter()
        .filter(|fk| !curr_fks.contains(&fk_key(fk)))
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

    // Enum : renames (même position, valeur différente), ajouts et suppressions de variantes
    let mut enum_renames: Vec<(String, String, String)> = Vec::new();
    let mut enum_value_adds: Vec<(String, String, String)> = Vec::new();
    let mut enum_value_drops: Vec<(String, String, String)> = Vec::new();

    for (name, curr) in &curr_cols {
        if curr.enum_string_values.is_empty() {
            continue;
        }
        if let Some(prev) = prev_cols.get(name) {
            let prev_set: HashSet<&str> =
                prev.enum_string_values.iter().map(|s| s.as_str()).collect();
            let curr_set: HashSet<&str> =
                curr.enum_string_values.iter().map(|s| s.as_str()).collect();

            // Valeurs ajoutées
            for v in curr_set.difference(&prev_set) {
                let enum_name = curr.enum_name.as_deref().unwrap_or(name).to_string();
                enum_value_adds.push((name.to_string(), enum_name, v.to_string()));
            }
            // Valeurs supprimées
            for v in prev_set.difference(&curr_set) {
                let enum_name = prev.enum_name.as_deref().unwrap_or(name).to_string();
                enum_value_drops.push((name.to_string(), enum_name, v.to_string()));
            }
            // Renames par position (parmi les valeurs présentes dans les deux)
            for (i, new_val) in curr.enum_string_values.iter().enumerate() {
                if let Some(old_val) = prev.enum_string_values.get(i) {
                    if old_val != new_val
                        && prev_set.contains(old_val.as_str())
                        && !curr_set.contains(old_val.as_str())
                    {
                        enum_renames.push((name.to_string(), old_val.clone(), new_val.clone()));
                    }
                }
            }
        }
    }

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
        enum_renames,
        enum_value_adds,
        enum_value_drops,
    }
}
