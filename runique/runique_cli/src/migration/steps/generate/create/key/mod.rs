//! A foreign key created: on an existing table (here), in an FK cycle, in a snapshot.
mod cycle_keys;
mod snapshot_key;

pub(crate) use cycle_keys::cycle_keys;
pub(crate) use snapshot_key::snapshot_key;

use crate::migration::types::ParsedFk;

/// The key `key` added to the existing table `table` under `name`, through
/// Runique (`migration::runtime`), which does it as the engine needs.
pub(crate) fn key(table: &str, name: &str, key: &ParsedFk) -> String {
    format!(
        "        runique_cli::migration::runtime::add_foreign_key(\n            manager,\n            \"{table}\",\n            ForeignKey::create()\n                .name(\"{fk_name}\")\n                .from(Alias::new(\"{table}\"), Alias::new(\"{from}\"))\n                .to(Alias::new(\"{to_table}\"), Alias::new(\"{to_col}\"))\n                .on_delete(ForeignKeyAction::{on_delete:?})\n                .on_update(ForeignKeyAction::{on_update:?})\n                .to_owned(),\n        )\n        .await?;\n\n",
        table = table,
        fk_name = name,
        from = key.from_column,
        to_table = key.to_table,
        to_col = key.to_column,
        on_delete = key.on_delete,
        on_update = key.on_update,
    )
}
