//! What follows a table rename in the snapshot a table is compared to.
use crate::migration::types::{ParsedFk, ParsedSchema, RenamedKey, TableRenames};

/// Brings `previous`, the snapshot `current` is compared to, in line with the
/// table renames of this run (`renames`: old name, new name), so the diff
/// only sees what the developer really changed. What it takes out of the
/// diff is returned: the table's own rename when the snapshot is the one of
/// its former name, the keys whose name holds a renamed table, and the
/// indexes named after the former table.
pub fn follow_renames(
    previous: &mut ParsedSchema,
    current: &ParsedSchema,
    renames: &[(String, String)],
) -> TableRenames {
    let old_table = std::mem::replace(&mut previous.table_name, current.table_name.clone());
    let renamed = old_table != current.table_name;
    let name_now = |table: &str| {
        renames
            .iter()
            .find(|(old, _)| old == table)
            .map_or(table, |(_, new)| new.as_str())
            .to_string()
    };

    // A key is renamed when it says the same thing once the renames are
    // applied. One that changed otherwise is left to the diff: removed under
    // its old name, added under its new one.
    let mut keys = Vec::new();
    for fk in &mut previous.foreign_keys {
        let target = name_now(&fk.to_table);
        if !renamed && target == fk.to_table {
            continue;
        }
        let now = ParsedFk {
            to_table: target,
            ..fk.clone()
        };
        if current.foreign_keys.contains(&now) {
            keys.push(RenamedKey {
                old_table: old_table.clone(),
                old_target: fk.to_table.clone(),
                target_at_run: now.to_table.clone(),
                key: now.clone(),
            });
            *fk = now;
        }
    }

    // An index of the former name with the same columns under another name.
    let mut indexes = Vec::new();
    if renamed {
        for at in 0..previous.indexes.len() {
            let old = previous.indexes[at].clone();
            if current.indexes.iter().any(|idx| idx.name == old.name) {
                continue;
            }
            let Some(new) = current.indexes.iter().find(|idx| {
                idx.columns == old.columns
                    && idx.unique == old.unique
                    && !previous.indexes.iter().any(|p| p.name == idx.name)
            }) else {
                continue;
            };
            previous.indexes[at].name = new.name.clone();
            indexes.push((old, new.clone()));
        }
    }

    TableRenames {
        from: renamed.then_some(old_table),
        keys,
        indexes,
    }
}
