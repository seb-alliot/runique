//! A foreign key renamed, through Runique (`migration::runtime`).
use crate::migration::{steps::generate::constraints::fk_constraint_name, types::ParsedFk};

/// The key `fk` of `table` takes the name of `named_for`'s tables in place of
/// `old_name`. It points to `target`, the name its target has at that moment.
pub(crate) fn key(
    table: &str,
    old_name: &str,
    named_for: (&str, &str),
    fk: &ParsedFk,
    target: &str,
) -> String {
    format!(
        "        runique_cli::migration::runtime::rename_foreign_key(\n            manager,\n            \"{table}\",\n            \"{old_name}\",\n            ForeignKey::create()\n                .name(\"{new_name}\")\n                .from(Alias::new(\"{table}\"), Alias::new(\"{from}\"))\n                .to(Alias::new(\"{target}\"), Alias::new(\"{to_col}\"))\n                .on_delete(ForeignKeyAction::{on_delete:?})\n                .on_update(ForeignKeyAction::{on_update:?})\n                .to_owned(),\n        )\n        .await?;\n\n",
        new_name = fk_constraint_name(named_for.0, &fk.from_column, named_for.1),
        from = fk.from_column,
        to_col = fk.to_column,
        on_delete = fk.on_delete,
        on_update = fk.on_update,
    )
}
