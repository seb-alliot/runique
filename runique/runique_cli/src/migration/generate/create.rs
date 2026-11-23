//! The columns of a CREATE TABLE, primary key included.
use super::columns::{col_type_to_method, render_column_def};
use super::constraints::fk_constraint_name;
use crate::migration::types::{ParsedColumn, ParsedSchema};

pub(super) fn build_create_table_cols(schema: &ParsedSchema, inline_fks: bool) -> String {
    let mut cols = String::new();

    if let Some(ref pk) = schema.primary_key {
        cols.push_str(&format!("{}\n", render_pk_col(pk)));
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
                "                    .foreign_key(\n                        ForeignKey::create()\n                            .name(\"{fk_name}\")\n                            .from(Alias::new(\"{table}\"), Alias::new(\"{from}\"))\n                            .to(Alias::new(\"{to_table}\"), Alias::new(\"{to_col}\"))\n                            .on_delete(ForeignKeyAction::{on_delete})\n                            .on_update(ForeignKeyAction::{on_update})\n                    )\n",
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

fn render_pk_col(pk: &ParsedColumn) -> String {
    let ty = col_type_to_method(&pk.col_type);

    // auto_increment only makes sense for integer-ish PK.
    let autoinc_ok = matches!(
        pk.col_type.as_str(),
        "Integer" | "BigInteger" | "SmallInteger" | "TinyInteger" | "Unsigned" | "BigUnsigned"
    );

    let mut s = format!(
        "                    .col(ColumnDef::new(Alias::new(\"{name}\")).{ty}.not_null()",
        name = pk.name,
        ty = ty
    );

    if autoinc_ok {
        s.push_str(".auto_increment()");
    }

    s.push_str(".primary_key())");
    s
}
