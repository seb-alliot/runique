//! Table renames (`renamed_from`): what follows a rename in the snapshot a
//! table is compared to, and where each renamed key points while it runs.
use crate::migration::types::{Changes, ParsedFk, ParsedSchema, RenamedKey, TableRenames};
use anyhow::Result;
use runique_i18n::tf;

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

/// Once `changes` are in the order their migrations run: a renamed key that
/// points to a table renamed later in the run names it as it still is, and
/// follows it when its turn comes. A key that is new to a renamed table can't
/// do that, its name would be wrong: pointing to a table this run creates, or
/// renames later, it is refused.
pub fn settle_renamed_keys(changes: &mut [Changes]) -> Result<()> {
    let renamed: Vec<(String, String)> = changes
        .iter()
        .filter_map(|c| Some((c.renames.from.clone()?, c.table_name.clone())))
        .collect();
    let created: Vec<String> = changes
        .iter()
        .filter(|c| c.is_new_table)
        .map(|c| c.table_name.clone())
        .collect();

    let mut done: Vec<String> = Vec::new();
    for change in changes.iter_mut() {
        if change.renames.from.is_some() {
            done.push(change.table_name.clone());
            let waits = |table: &String| {
                created.contains(table)
                    || (renamed.iter().any(|(_, new)| new == table) && !done.contains(table))
            };
            if let Some(fk) = change.added_fks.iter().find(|fk| waits(&fk.to_table)) {
                anyhow::bail!(
                    "{}",
                    tf(
                        runique_i18n::keys::makemigrations::error::RENAME_WITH_NEW_KEY,
                        &[&change.table_name, &fk.to_table]
                    )
                );
            }
        }
        for key in &mut change.renames.keys {
            if let Some((old, new)) = renamed.iter().find(|(_, new)| *new == key.key.to_table)
                && !done.contains(new)
            {
                key.target_at_run = old.clone();
            }
        }
    }
    Ok(())
}
