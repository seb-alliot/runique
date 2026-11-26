use super::fk_constraint_name;
use crate::migration::types::ParsedFk;

/// `ForeignKey::create()…` for the key `fk` of `table`, each line indented by `indent`.
pub(crate) fn render_fk_create(table: &str, fk: &ParsedFk, indent: &str) -> String {
    format!(
        "{indent}ForeignKey::create()\n{indent}    .name(\"{name}\")\n{indent}    .from(Alias::new(\"{table}\"), Alias::new(\"{from}\"))\n{indent}    .to(Alias::new(\"{to_table}\"), Alias::new(\"{to_col}\"))\n{indent}    .on_delete(ForeignKeyAction::{on_delete:?})\n{indent}    .on_update(ForeignKeyAction::{on_update:?})",
        name = fk_constraint_name(table, &fk.from_column, &fk.to_table),
        from = fk.from_column,
        to_table = fk.to_table,
        to_col = fk.to_column,
        on_delete = fk.on_delete,
        on_update = fk.on_update,
    )
}
