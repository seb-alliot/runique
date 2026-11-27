//! The keys of a cycle between tables created together.
use crate::migration::*;

/// For each new table, the foreign keys of an FK cycle: keys to a new table
/// created later in this batch (`forward`), and keys pointing to it from a
/// table created earlier (`closing`). `all_changes` is in creation order.
pub fn cycle_keys<'a>(
    all_changes: &[Changes],
    schemas: &'a [ParsedSchema],
) -> std::collections::HashMap<&'a str, CycleKeys<'a>> {
    use std::collections::{HashMap, HashSet};

    let new_tables: HashSet<&str> = all_changes
        .iter()
        .filter(|c| c.table == TableChange::Created)
        .map(|c| c.table_name.as_str())
        .collect();
    let mut created: HashSet<&str> = HashSet::new();
    let mut keys: HashMap<&'a str, CycleKeys<'a>> = HashMap::new();
    for change in all_changes
        .iter()
        .filter(|c| c.table == TableChange::Created)
    {
        let Some(schema) = schemas.iter().find(|s| s.table_name == change.table_name) else {
            continue;
        };
        let table = schema.table_name.as_str();
        for fk in &schema.foreign_keys {
            let target = fk.to_table.as_str();
            if target != table && new_tables.contains(target) && !created.contains(target) {
                keys.entry(table).or_default().forward.push(fk);
                if let Some(target) = schemas.iter().find(|s| s.table_name == target) {
                    keys.entry(target.table_name.as_str())
                        .or_default()
                        .closing
                        .push((table, fk));
                }
            }
        }
        created.insert(table);
    }
    keys
}
