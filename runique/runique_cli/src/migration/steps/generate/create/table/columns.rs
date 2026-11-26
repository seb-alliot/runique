use super::primary_key::primary_key;
use crate::migration::steps::generate::columns::render_column_def;
use crate::migration::steps::generate::constraints::fk_constraint_name;
use crate::migration::types::ParsedSchema;

/// The columns of a CREATE TABLE, primary key included.
pub(super) fn columns(schema: &ParsedSchema, inline_fks: bool) -> String {
    let mut cols = String::new();

    if let Some(ref pk) = schema.primary_key {
        cols.push_str(&format!("{}\n", primary_key(pk)));
    }

    for col in schema.columns.iter().filter(|c| !c.ignored) {
        cols.push_str(&format!(
            "                    .col({})\n",
            render_column_def(col)
        ));
    }

    // SQLite cannot add FK constraints to an existing table (no ALTER TABLE ADD CONSTRAINT),
    // so for SQLite the FKs are declared INLINE here instead of in the relations migration.
    if inline_fks {
        for fk in &schema.foreign_keys {
            cols.push_str(&format!(
                "                    .foreign_key(\n                        ForeignKey::create()\n                            .name(\"{fk_name}\")\n                            .from(Alias::new(\"{table}\"), Alias::new(\"{from}\"))\n                            .to(Alias::new(\"{to_table}\"), Alias::new(\"{to_col}\"))\n                            .on_delete(ForeignKeyAction::{on_delete:?})\n                            .on_update(ForeignKeyAction::{on_update:?})\n                    )\n",
                fk_name = fk_constraint_name(&schema.table_name, &fk.from_column, &fk.to_table),
                table = schema.table_name,
                from = fk.from_column,
                to_table = fk.to_table,
                to_col = fk.to_column,
                on_delete = fk.on_delete,
                on_update = fk.on_update,
            ));
        }
    }

    cols
}
