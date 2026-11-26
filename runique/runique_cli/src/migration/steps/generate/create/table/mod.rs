//! A table created, with its columns, its primary key and its inline keys.
mod columns;
mod primary_key;

use crate::migration::steps::generate::constraints::render_fk_create;
use crate::migration::steps::generate::instruction::File;
use crate::migration::types::{ParsedFk, ParsedSchema};

/// `forward`: the table's keys to tables created after it, in an FK cycle.
/// In a migration the other keys are written inside the CREATE TABLE; a
/// snapshot keeps them apart.
pub(crate) fn table(schema: &ParsedSchema, forward: &[&ParsedFk], file: File) -> String {
    // FKs inline in CREATE TABLE: SQLite cannot ALTER-ADD a foreign key to an
    // existing table. Only the forward keys of a cycle wait on other engines.
    let inline = ParsedSchema {
        foreign_keys: schema
            .foreign_keys
            .iter()
            .filter(|fk| !forward.contains(fk))
            .cloned()
            .collect(),
        ..schema.clone()
    };
    let cols = columns::columns(&inline, matches!(file, File::Migration));

    let mut up = String::new();
    if forward.is_empty() {
        up.push_str("        manager\n");
        up.push_str("            .create_table(\n");
        up.push_str("                Table::create()\n");
        up.push_str(&format!(
            "                    .table(Alias::new(\"{}\"))\n",
            schema.table_name
        ));
        up.push_str("                    .if_not_exists()\n");
        up.push_str(&cols);
        up.push_str("                    .to_owned()\n");
        up.push_str("            )\n");
        up.push_str("            .await?;\n\n");
    } else {
        up.push_str("        let mut table = Table::create();\n");
        up.push_str("        table\n");
        up.push_str(&format!(
            "                    .table(Alias::new(\"{}\"))\n",
            schema.table_name
        ));
        up.push_str("                    .if_not_exists()\n");
        up.push_str(cols.trim_end());
        up.push_str(";\n");
        up.push_str(
            "        // FK cycle: the target table doesn't exist yet. SQLite takes the key\n",
        );
        up.push_str("        // now; Postgres/MySQL get it once the target is created.\n");
        up.push_str("        if manager.get_connection().get_database_backend() == sea_orm::DbBackend::Sqlite {\n");
        for fk in forward {
            up.push_str(&format!(
                "            table.foreign_key(\n{});\n",
                render_fk_create(&schema.table_name, fk, "                ")
            ));
        }
        up.push_str("        }\n");
        up.push_str("        manager.create_table(table.to_owned()).await?;\n\n");
    }
    up
}
