//! The order tables are created in: by foreign-key dependency.
use crate::migration::types::TableChange;

/// Sorts `Changes` by FK dependency order: a new table comes after the new
/// tables it references, ties broken by name so the order never depends on a
/// hash map. A cycle of new tables is broken at its smallest name; the foreign
/// keys left pointing forward are added once their target exists (see
/// `build_main_plan`). Changes to existing tables follow, by name. Renamed
/// tables go before everything, by name: what is created or altered after
/// them names them as they are called now.
pub fn topological_sort_changes(
    changes: Vec<crate::migration::types::Changes>,
) -> Vec<crate::migration::types::Changes> {
    use std::collections::{BTreeMap, BTreeSet};

    let new_tables: BTreeSet<String> = changes
        .iter()
        .filter(|c| c.table == TableChange::Created)
        .map(|c| c.table_name.clone())
        .collect();

    // waits_for[A] = new tables A references (not yet placed)
    let mut waits_for: BTreeMap<String, BTreeSet<String>> = new_tables
        .iter()
        .map(|t| (t.clone(), BTreeSet::new()))
        .collect();
    for change in changes.iter().filter(|c| c.table == TableChange::Created) {
        for fk in &change.added_fks {
            if new_tables.contains(&fk.to_table) && fk.to_table != change.table_name {
                waits_for
                    .entry(change.table_name.clone())
                    .or_default()
                    .insert(fk.to_table.clone());
            }
        }
    }

    let mut sorted_names: Vec<String> = Vec::with_capacity(new_tables.len());
    while !waits_for.is_empty() {
        let next = waits_for
            .iter()
            .find(|(_, deps)| deps.is_empty())
            .or_else(|| waits_for.iter().next())
            .map(|(t, _)| t.clone())
            .expect("waits_for is not empty");
        waits_for.remove(&next);
        for deps in waits_for.values_mut() {
            deps.remove(&next);
        }
        sorted_names.push(next);
    }

    let mut by_name: BTreeMap<String, crate::migration::types::Changes> = changes
        .into_iter()
        .map(|c| (c.table_name.clone(), c))
        .collect();
    let mut result: Vec<crate::migration::types::Changes> = Vec::with_capacity(by_name.len());
    let renamed: Vec<String> = by_name
        .values()
        .filter(|c| c.renames.from.is_some())
        .map(|c| c.table_name.clone())
        .collect();
    for name in renamed.into_iter().chain(sorted_names) {
        if let Some(c) = by_name.remove(&name) {
            result.push(c);
        }
    }
    result.extend(by_name.into_values());
    result
}
