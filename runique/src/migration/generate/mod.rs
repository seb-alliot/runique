//! SeaORM migration Rust code generation — `up`/`down` files, CREATE TABLE, FK, indexes, triggers.
//! The four entry points live here; each kind of statement has its own file.
mod alter;
mod columns;
mod constraints;
mod create;
mod enums;

pub use columns::{col_type_method, col_type_to_method};

use crate::migration::types::{Changes, ParsedFk, ParsedSchema};
use alter::{body_has_code, build_alter_bodies};
use constraints::{
    build_fk_create_stmts, build_fk_drop_stmts, build_index_create_stmts, build_index_drop_stmts,
    fk_constraint_name, render_fk_create,
};
use create::build_create_table_cols;
use enums::{
    build_enum_create_stmts_for_cols, build_enum_drop_stmts_for_cols, build_enum_type_drops,
    build_enum_type_stmts,
};

/// Generates the migration file for a CREATE TABLE.
pub fn generate_create_file(schema: &ParsedSchema) -> String {
    generate_create_file_in_cycle(schema, &CycleKeys::default())
}

/// Foreign keys of an FK cycle between new tables: Postgres and MySQL refuse
/// a key to a table that doesn't exist yet, so these are left out of their
/// table's CREATE and added once the target exists. SQLite only checks keys
/// when rows are written, so there they stay inline — it can't add one later.
#[derive(Debug, Default)]
pub struct CycleKeys<'a> {
    /// This table's keys to tables created after it.
    pub forward: Vec<&'a ParsedFk>,
    /// Keys of tables created before this one that point to it: (their table, key).
    pub closing: Vec<(&'a str, &'a ParsedFk)>,
}

const NOT_SQLITE: &str =
    "manager.get_connection().get_database_backend() != sea_orm::DbBackend::Sqlite";

/// The CREATE file of a table, with the keys of an FK cycle placed per engine.
pub fn generate_create_file_in_cycle(schema: &ParsedSchema, keys: &CycleKeys<'_>) -> String {
    let (forward, closing) = (&keys.forward, &keys.closing);

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
    let cols = build_create_table_cols(&inline, true);
    let idx_stmts = build_index_create_stmts(schema, true);
    let enum_stmts = build_enum_type_stmts(schema);
    let enum_drops = build_enum_type_drops(schema);

    let mut up = String::new();
    up.push_str(&enum_stmts);
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
    up.push_str(&idx_stmts);
    if !closing.is_empty() {
        up.push_str(&format!("        if {NOT_SQLITE} {{\n"));
        for (from_table, fk) in closing {
            up.push_str(&format!(
                "            manager\n                .create_foreign_key(\n{}\n                        .to_owned(),\n                )\n                .await?;\n",
                render_fk_create(from_table, fk, "                    ")
            ));
        }
        up.push_str("        }\n");
    }
    up.push_str("        Ok(())\n");

    let mut down = String::new();
    if !closing.is_empty() {
        down.push_str(&format!("        if {NOT_SQLITE} {{\n"));
        for (from_table, fk) in closing {
            down.push_str(&format!(
                "            manager\n                .drop_foreign_key(\n                    ForeignKey::drop()\n                        .table(Alias::new(\"{from_table}\"))\n                        .name(\"{name}\")\n                        .to_owned(),\n                )\n                .await?;\n",
                name = fk_constraint_name(from_table, &fk.from_column, &fk.to_table),
            ));
        }
        down.push_str("        }\n");
    }
    // No explicit drop_index here: DROP TABLE already removes every index defined on it,
    // and dropping one by name first fails on MariaDB/MySQL when it's still backing an
    // active FK constraint (error 1553) — the FK's own drop only happens implicitly with
    // the table, never explicitly before this point in a CREATE-table migration's down().
    down.push_str("        manager\n");
    down.push_str("            .drop_table(Table::drop()\n");
    down.push_str(&format!(
        "                .table(Alias::new(\"{}\"))\n",
        schema.table_name
    ));
    down.push_str("                .to_owned())\n");
    down.push_str("            .await?;\n");
    down.push_str(&enum_drops);
    down.push_str("        Ok(())\n");

    format!(
        "use sea_orm_migration::prelude::*;\n\n\
    #[derive(DeriveMigrationName)]\n\
    pub struct Migration;\n\n\
    #[async_trait::async_trait]\n\
    impl MigrationTrait for Migration {{\n\
        async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {{\n\
    {up}\
        }}\n\n\
        async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {{\n\
    {down}\
        }}\n\
    }}\n",
        up = up,
        down = down,
    )
}

/// First line of every snapshot that records column lengths. A snapshot
/// without it predates them: its `None` lengths mean "unknown", not "none".
pub const SNAPSHOT_LENGTHS_MARKER: &str = "// runique: column lengths recorded";

/// Generates the snapshot file (includes FK stmts so diffs detect FK additions/removals).
pub fn generate_snapshot_file(schema: &ParsedSchema) -> String {
    // Snapshot keeps FKs as separate stmts (never inline) so the `parse/seaorm.rs` round-trip is stable.
    let cols = build_create_table_cols(schema, false);
    let fk_stmts = build_fk_create_stmts(schema);
    let idx_stmts = build_index_create_stmts(schema, false);
    let fk_drops = build_fk_drop_stmts(schema);
    let idx_drops = build_index_drop_stmts(schema);

    let mut up = String::new();
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
    up.push_str(&fk_stmts);
    up.push_str(&idx_stmts);
    up.push_str("        Ok(())\n");

    let mut down = String::new();
    down.push_str(&fk_drops);
    down.push_str(&idx_drops);
    down.push_str("        manager\n");
    down.push_str("            .drop_table(Table::drop()\n");
    down.push_str(&format!(
        "                .table(Alias::new(\"{}\"))\n",
        schema.table_name
    ));
    down.push_str("                .to_owned())\n");
    down.push_str("            .await?;\n");
    down.push_str("        Ok(())\n");

    format!(
        "{SNAPSHOT_LENGTHS_MARKER}\n\
    use sea_orm_migration::prelude::*;\n\n\
    #[derive(DeriveMigrationName)]\n\
    pub struct Migration;\n\n\
    #[async_trait::async_trait]\n\
    impl MigrationTrait for Migration {{\n\
        async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {{\n\
    {up}\
        }}\n\n\
        async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {{\n\
    {down}\
        }}\n\
    }}\n",
        up = up,
        down = down,
    )
}

/// Generates a standalone `ALTER TABLE` migration file from a single [`Changes`] set
/// (renames, add/drop columns, FKs, indexes, enum transitions, etc.), each op emitted
/// in dependency-safe order in both `up` and `down`.
pub fn generate_alter_file(change: &Changes) -> String {
    let (up_body, down_body) = build_alter_bodies(change);

    let enum_creates = build_enum_create_stmts_for_cols(&change.added_columns);
    let enum_drops = build_enum_drop_stmts_for_cols(&change.added_columns);

    let full_up = format!("{}{}", enum_creates, up_body);
    let full_down = format!("{}{}", down_body, enum_drops);

    let up_param = if body_has_code(&full_up) {
        "manager"
    } else {
        "_manager"
    };
    let down_param = if body_has_code(&full_down) {
        "manager"
    } else {
        "_manager"
    };

    format!(
        "use sea_orm_migration::prelude::*;\n\n#[derive(DeriveMigrationName)]\npub struct Migration;\n\n#[async_trait::async_trait]\nimpl MigrationTrait for Migration {{\n    async fn up(&self, {up_param}: &SchemaManager) -> Result<(), DbErr> {{\n{up}\n        Ok(())\n    }}\n\n    async fn down(&self, {down_param}: &SchemaManager) -> Result<(), DbErr> {{\n{down}\n        Ok(())\n    }}\n}}\n",
        up_param = up_param,
        down_param = down_param,
        up = full_up.trim_end(),
        down = full_down.trim_end()
    )
}

fn indent(code: &str, prefix: &str) -> String {
    code.lines()
        .map(|l| {
            if l.is_empty() {
                "\n".to_string()
            } else {
                format!("{prefix}{l}\n")
            }
        })
        .collect()
}
