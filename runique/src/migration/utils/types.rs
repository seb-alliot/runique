//! Shared data types between migration utilities: parsed schemas, columns, FKs, indexes, diffs.

/// Target database backend — used to generate DB-specific SQL
/// (e.g., ON UPDATE CURRENT_TIMESTAMP for MySQL, trigger for PostgreSQL).
#[derive(Debug, Clone, PartialEq, Default)]
pub enum DbKind {
    Postgres,
    Mysql,
    #[default]
    Other,
}

#[derive(Debug, Clone)]
pub struct ParsedSchema {
    pub table_name: String,
    pub primary_key: Option<ParsedColumn>,
    pub columns: Vec<ParsedColumn>,
    pub foreign_keys: Vec<ParsedFk>,
    pub indexes: Vec<ParsedIndex>,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct ParsedColumn {
    pub name: String,
    pub col_type: String,
    pub nullable: bool,
    pub unique: bool,
    pub ignored: bool,
    pub created_at: bool,
    pub updated_at: bool,
    /// Column with DEFAULT CURRENT_TIMESTAMP — detected from the builder or SeaORM snapshot.
    pub has_default_now: bool,
    /// Literal default value (`[default: 0]`, `[default: true]`, `[default: "x"]`) rendered as
    /// the Rust expression placed inside `.default(...)`. `None` when no literal default.
    /// Distinct from `has_default_now` (timestamp default), which remains its own flag.
    pub default_value: Option<String>,
    /// Enum name for string enum columns (used in the snapshot).
    pub enum_name: Option<String>,
    /// Current DB values for string enum columns (e.g., ["Fix", "Feature", "Added"]).
    pub enum_string_values: Vec<String>,
    pub enum_is_pg: bool,
    /// Explicit rename directive (`[renamed_from: "old_name"]`). Transient: it lives in the
    /// source model only and is NEVER written to snapshots (the snapshot holds the new name).
    /// Consumed by the diff to emit `RENAME COLUMN` instead of DROP + ADD (no data loss).
    pub renamed_from: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ParsedFk {
    pub from_column: String,
    pub to_table: String,
    pub to_column: String,
    pub on_delete: String,
    pub on_update: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ParsedIndex {
    pub name: String,
    pub columns: Vec<String>,
    pub unique: bool,
}

#[derive(Debug, Clone)]
pub struct Changes {
    pub table_name: String,
    pub added_columns: Vec<ParsedColumn>,
    pub dropped_columns: Vec<ParsedColumn>, // <- CHANGED
    pub modified_columns: Vec<(ParsedColumn, ParsedColumn)>,
    /// Column renames via `[renamed_from: "old"]`: (old_name, new_name). Emitted as
    /// `ALTER TABLE … RENAME COLUMN` (PG/MySQL/MariaDB/SQLite all support it).
    pub renamed_columns: Vec<(String, String)>,
    pub added_fks: Vec<ParsedFk>,
    pub dropped_fks: Vec<ParsedFk>,
    pub added_indexes: Vec<ParsedIndex>,
    pub dropped_indexes: Vec<ParsedIndex>,
    pub is_new_table: bool,
    /// String enum value renames: (column_name, enum_name, old_value, new_value).
    pub enum_renames: Vec<(String, String, String, String)>,
    /// Added enum values: (column_name, value).
    pub enum_value_adds: Vec<(String, String, String)>,
    /// Dropped enum values: (column_name, value).
    pub enum_value_drops: Vec<(String, String, String)>,
}

impl Changes {
    pub fn is_empty(&self) -> bool {
        !self.is_new_table
            && self.added_columns.is_empty()
            && self.dropped_columns.is_empty()
            && self.modified_columns.is_empty()
            && self.renamed_columns.is_empty()
            && self.added_fks.is_empty()
            && self.dropped_fks.is_empty()
            && self.added_indexes.is_empty()
            && self.dropped_indexes.is_empty()
            && self.enum_renames.is_empty()
            && self.enum_value_adds.is_empty()
            && self.enum_value_drops.is_empty()
    }
}
