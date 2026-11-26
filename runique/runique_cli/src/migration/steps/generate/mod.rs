//! SeaORM migration Rust code generation — `up`/`down` files, CREATE TABLE, FK, indexes, triggers.
//! The four entry points live here. A migration body is a list of
//! instructions (`instruction`): an action (`create`, `delete`, `modify`,
//! `rename`) on an object, each case written by its own function.
mod alter;
mod columns;
mod constraints;
mod create;
mod delete;
pub(crate) mod init_migrate;
mod instruction;
pub(crate) mod lib_file;
mod modify;
pub(crate) mod plan;
mod rename;

pub use columns::col_type_method;
pub use lib_file::{ensure_admin_migration_positioned, update_migration_lib};

use crate::migration::types::{Changes, ParsedColumn, ParsedFk, ParsedSchema};
use alter::{body_has_code, build_alter_bodies};
use constraints::fk_constraint_name;
use constraints::key_changes_guard;
use instruction::{Create, Delete, File, Instruction, chain};

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
    let (up, down) = create_bodies(schema, keys, false);
    migration_file(&up, &down)
}

/// The migration that drops a table: the mirror of its creation. `up` takes
/// the table away (`IF EXISTS`: a table already gone must not fail the run),
/// after the keys that still point to it from tables dropped later, then the
/// enum types nothing uses any more. `down` creates it again from `schema`,
/// its snapshot: the table comes back, empty.
pub fn generate_drop_file(schema: &ParsedSchema, keys: &CycleKeys<'_>) -> String {
    let (create, drop) = create_bodies(schema, keys, true);
    migration_file(&drop, &create)
}

/// The two bodies around a table: what creates it, and what takes it away.
fn create_bodies(
    schema: &ParsedSchema,
    keys: &CycleKeys<'_>,
    drop_if_exists: bool,
) -> (String, String) {
    let table = schema.table_name.as_str();
    let enums = enum_types(&schema.columns);

    let mut up: Vec<Instruction> = Vec::new();
    for (name, values) in &enums {
        up.push(Instruction::Create(Create::EnumType { name, values }));
    }
    up.push(Instruction::Create(Create::Table {
        schema,
        forward: &keys.forward,
        file: File::Migration,
    }));
    for index in &schema.indexes {
        up.push(Instruction::Create(Create::Index {
            table,
            named_for: table,
            index,
            file: File::Migration,
        }));
    }
    if !keys.closing.is_empty() {
        up.push(Instruction::Create(Create::CycleKeys {
            keys: &keys.closing,
        }));
    }

    let mut down: Vec<Instruction> = Vec::new();
    if !keys.closing.is_empty() {
        down.push(Instruction::Delete(Delete::CycleKeys {
            keys: &keys.closing,
        }));
    }
    // No explicit drop_index here: DROP TABLE already removes every index defined on it,
    // and dropping one by name first fails on MySQL when it's still backing an
    // active FK constraint (error 1553) — the FK's own drop only happens implicitly with
    // the table, never explicitly before this point in a CREATE-table migration's down().
    down.push(Instruction::Delete(Delete::Table {
        name: table,
        if_exists: drop_if_exists,
    }));
    for (name, values) in &enums {
        down.push(Instruction::Delete(Delete::EnumType { name, values }));
    }

    (
        format!("{}        Ok(())\n", chain(&up)),
        format!("{}        Ok(())\n", chain(&down)),
    )
}

/// The enum types `columns` need, as (name, values): one per enum column.
fn enum_types(columns: &[ParsedColumn]) -> Vec<(&str, &[String])> {
    columns
        .iter()
        .filter(|column| !column.enum_string_values.is_empty())
        .map(|column| {
            (
                column.enum_name.as_deref().unwrap_or(&column.name),
                column.enum_string_values.as_slice(),
            )
        })
        .collect()
}

/// A migration file around its two bodies.
fn migration_file(up: &str, down: &str) -> String {
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

/// First line of generated migration that no `migration up` applied yet.
pub const WAIT_MIGRATION: &str = "// runique: waiting migrate up";

/// Start of the line a generated migration carries under its tag: the
/// migration it starts from for its table, or `none` for the first one. Two
/// migrations of a table starting from the same one were written in parallel.
pub const FROM_MIGRATION: &str = "// runique: from ";

/// What a generated migration says it starts from; `None` when it doesn't say
/// (written by hand, or before that line existed).
pub fn started_from(content: &str) -> Option<&str> {
    content
        .lines()
        .find_map(|line| line.trim().strip_prefix(FROM_MIGRATION))
        .map(str::trim)
}

/// Start of the line a migration that renames a table carries: the name the
/// table had. `migration up` reads it to take the snapshot of that name away.
pub const RENAMED_TABLE: &str = "// runique: renamed table ";

/// The former name of the table a migration renames; `None` when it renames
/// none, or when what follows the marker isn't a plain table name (the name
/// becomes a file path).
pub fn renamed_table(content: &str) -> Option<&str> {
    content
        .lines()
        .find_map(|line| line.trim().strip_prefix(RENAMED_TABLE))
        .map(str::trim)
        .filter(|name| {
            !name.is_empty() && name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
        })
}

/// Generates the snapshot file (includes FK stmts so diffs detect FK additions/removals).
pub fn generate_snapshot_file(schema: &ParsedSchema) -> String {
    let table = schema.table_name.as_str();
    // Snapshot keeps FKs as separate stmts (never inline) so the `parser/seaorm.rs` round-trip is stable.
    let mut up = vec![Instruction::Create(Create::Table {
        schema,
        forward: &[],
        file: File::Snapshot,
    })];
    let mut down: Vec<Instruction> = Vec::new();
    for key in &schema.foreign_keys {
        let name = fk_constraint_name(table, &key.from_column, &key.to_table);
        up.push(Instruction::Create(Create::Key {
            table,
            name: name.clone(),
            key,
            file: File::Snapshot,
        }));
        down.push(Instruction::Delete(Delete::Key {
            table,
            name,
            key,
            file: File::Snapshot,
        }));
    }
    for index in &schema.indexes {
        up.push(Instruction::Create(Create::Index {
            table,
            named_for: table,
            index,
            file: File::Snapshot,
        }));
        down.push(Instruction::Delete(Delete::Index {
            table,
            named_for: table,
            index,
            file: File::Snapshot,
        }));
    }
    down.push(Instruction::Delete(Delete::Table {
        name: table,
        if_exists: false,
    }));
    let up = format!("{}        Ok(())\n", chain(&up));
    let down = format!("{}        Ok(())\n", chain(&down));

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

    // Before anything else: an engine that can't change the keys of an
    // existing table stops here, the table untouched.
    let guard = if change.added_fks.is_empty() && change.dropped_fks.is_empty() {
        String::new()
    } else {
        key_changes_guard(&change.table_name)
    };
    let full_up = format!("{}{}", guard, up_body);
    let full_down = format!("{}{}", guard, down_body);

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
