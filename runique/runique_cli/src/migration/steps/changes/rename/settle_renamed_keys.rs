//! Where each renamed key points while its migration runs.
use crate::migration::types::{Changes, TableChange};
use anyhow::Result;
use runique_i18n::tf;

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
        .filter(|c| c.table == TableChange::Created)
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
