use crate::migration::types::ParsedFk;

/// A key as a snapshot keeps it: a statement of its own, never inline.
pub(crate) fn snapshot_key(table: &str, name: &str, key: &ParsedFk) -> String {
    format!(
        "        manager\n            .create_foreign_key(\n                ForeignKey::create()\n                    .name(\"{fk_name}\")\n                    .from(Alias::new(\"{table}\"), Alias::new(\"{from}\"))\n                    .to(Alias::new(\"{to_table}\"), Alias::new(\"{to_col}\"))\n                    .on_delete(ForeignKeyAction::{on_delete:?})\n                    .on_update(ForeignKeyAction::{on_update:?})\n                    .to_owned(),\n            )\n            .await?;\n\n",
        fk_name = name,
        table = table,
        from = key.from_column,
        to_table = key.to_table,
        to_col = key.to_column,
        on_delete = key.on_delete,
        on_update = key.on_update
    )
}
