//! A foreign key removed: on an existing table (here), in an FK cycle, in a snapshot.
mod cycle_keys;
mod snapshot_key;

pub(crate) use cycle_keys::cycle_keys;
pub(crate) use snapshot_key::snapshot_key;

/// The key `name` removed from the existing table `table`, through Runique
/// (`migration::runtime`), which does it as the engine needs.
pub(crate) fn key(table: &str, name: &str) -> String {
    format!(
        "        runique_cli::migration::runtime::drop_foreign_key(manager, \"{table}\", \"{fk_name}\")\n            .await?;\n\n",
        table = table,
        fk_name = name,
    )
}
