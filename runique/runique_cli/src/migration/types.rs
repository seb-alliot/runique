//! Shared data types between migration utilities: parsed schemas, columns, FKs, indexes, diffs.

/// What a foreign key does when the row it points to is deleted or updated.
pub use runique_dsl::ast::FkAction;

/// The type of a column in the database: the closed list of what a
/// migration can hold.
pub use runique_dsl::ColumnType;

/// A table schema as reconstructed by the AST parsers (from a `model!{}` DSL
/// source or a generated SeaORM snapshot). Feeds the diff engine that decides
/// what a migration needs to change.
#[derive(Debug, Clone, Default)]
pub struct ParsedSchema {
    pub table_name: String,
    pub primary_key: Option<ParsedColumn>,
    pub columns: Vec<ParsedColumn>,
    pub foreign_keys: Vec<ParsedFk>,
    pub indexes: Vec<ParsedIndex>,
}

/// A single parsed column, as extracted from either the `model!{}` DSL or a
/// generated SeaORM snapshot. Compared field-by-field between two schema
/// versions to build a [`Changes`] set.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ParsedColumn {
    pub name: String,
    pub col_type: ColumnType,
    pub nullable: bool,
    pub unique: bool,
    pub ignored: bool,
    /// Column with DEFAULT CURRENT_TIMESTAMP: declared `auto_now`/`auto_now_update`,
    /// or read back from a snapshot.
    pub has_default_now: bool,
    /// Literal default value (`[default: 0]`, `[default: true]`, `[default: "x"]`) rendered as
    /// the Rust expression placed inside `.default(...)`. `None` when no literal default.
    /// Distinct from `has_default_now` (timestamp default), which remains its own flag.
    pub default_value: Option<String>,
    /// Enum name for string enum columns (used in the snapshot).
    pub enum_name: Option<String>,
    /// Current DB values for string enum columns (e.g., ["Fix", "Feature", "Added"]).
    pub enum_string_values: Vec<String>,
    /// Explicit rename directive (`[renamed_from: "old_name"]`). Transient: it lives in the
    /// source model only and is NEVER written to snapshots (the snapshot holds the new name).
    /// Consumed by the diff to emit `RENAME COLUMN` instead of DROP + ADD (no data loss).
    pub renamed_from: Option<String>,
    /// Declared column length: `VARCHAR(n)` for text, `BINARY(n)`/`VARBINARY(n)`
    /// for bytes. `None` leaves the engine's default length.
    pub max_length: Option<u32>,
    /// A decimal's `(max_digits, decimal_places)`: `DECIMAL(p, s)`.
    pub decimal_len: Option<(u32, u32)>,
}

/// An enum the database knows by its values, declared once in a model or an
/// `extend!{}` block and used by one or several columns. An enum stored as
/// an integer is not one of them: the database only sees a number.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ParsedEnum {
    /// The name the database gives it: the declared one in lowercase, which
    /// is the name of its type under PostgreSQL.
    pub name: String,
    /// The values as they are stored, in the order they are declared in.
    pub values: Vec<String>,
    /// Where it is used. Read from the models: a snapshot doesn't hold them.
    pub places: Vec<EnumPlace>,
}

/// A column an enum is used by.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnumPlace {
    pub table: String,
    pub column: String,
}

/// What a run does to an enum as a whole: it creates it, gives it another
/// name, or leaves it in place and changes its values.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum EnumChange {
    Created,
    /// The enum the database has under [`EnumChanges::renamed_from`] takes
    /// the name the models give it now. Its values are another change.
    Renamed,
    #[default]
    Altered,
}

/// What brings an enum from its snapshot to what the models declare: the
/// output of the diff for an enum, as [`Changes`] is for a table.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct EnumChanges {
    pub name: String,
    /// What happens to the enum itself.
    pub change: EnumChange,
    /// The name the database has it under, when the run renames it.
    pub renamed_from: Option<String>,
    /// Its values as the models declare them.
    pub values: Vec<String>,
    /// (old value, new value).
    pub renamed_values: Vec<(String, String)>,
    pub added_values: Vec<String>,
    pub dropped_values: Vec<String>,
    /// Every column that uses the enum, as the models name them.
    pub places: Vec<EnumPlace>,
}

impl EnumChanges {
    /// True when there is nothing to migrate: an enum that exists already
    /// and whose values are the ones of its snapshot.
    pub fn is_empty(&self) -> bool {
        self.change == EnumChange::Altered
            && self.renamed_values.is_empty()
            && self.added_values.is_empty()
            && self.dropped_values.is_empty()
    }
}

/// A parsed foreign key constraint.
#[derive(Debug, Clone, PartialEq)]
pub struct ParsedFk {
    pub from_column: String,
    pub to_table: String,
    pub to_column: String,
    pub on_delete: FkAction,
    pub on_update: FkAction,
}

/// A parsed table index.
#[derive(Debug, Clone, PartialEq)]
pub struct ParsedIndex {
    pub name: String,
    pub columns: Vec<String>,
    pub unique: bool,
}

/// A foreign key that says what it said and changes its name, because the
/// table it belongs to or the table it points to was renamed: constraint
/// names hold both table names.
#[derive(Debug, Clone, PartialEq)]
pub struct RenamedKey {
    /// The key's table and target when the key got its current name.
    pub old_table: String,
    pub old_target: String,
    /// The key as the model declares it now.
    pub key: ParsedFk,
    /// The target's name at the moment this migration runs: still its old one
    /// when the target is renamed later in the same run. The key follows the
    /// table once it is renamed.
    pub target_at_run: String,
}

/// What a table rename (`renamed_from` after `table:`, or on a list field)
/// brings to a table: its own rename, and the keys and indexes whose name
/// follows it. A table that only points to a renamed one has `keys` alone.
#[derive(Debug, Clone, Default)]
pub struct TableRenames {
    /// The table's former name.
    pub from: Option<String>,
    pub keys: Vec<RenamedKey>,
    /// (as it is in the database, as it is named now)
    pub indexes: Vec<(ParsedIndex, ParsedIndex)>,
}

impl TableRenames {
    pub fn is_empty(&self) -> bool {
        self.from.is_none() && self.keys.is_empty() && self.indexes.is_empty()
    }
}

/// What a run does to a table as a whole: it creates it, drops it, or
/// leaves it in place and changes what it holds.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TableChange {
    Created,
    #[default]
    Altered,
    Deleted,
}

/// The set of DDL operations needed to bring a table from one [`ParsedSchema`]
/// version to the next — the output of the diff step, consumed by the
/// migration file generators to render `up`/`down` bodies.
#[derive(Debug, Clone, Default)]
pub struct Changes {
    pub table_name: String,
    pub added_columns: Vec<ParsedColumn>,
    pub dropped_columns: Vec<ParsedColumn>, // <- CHANGED
    pub modified_columns: Vec<(ParsedColumn, ParsedColumn)>,
    /// Column renames via `[renamed_from: "old"]`: (old_name, new_name). Emitted as
    /// `ALTER TABLE … RENAME COLUMN` (PG/MySQL/SQLite all support it).
    pub renamed_columns: Vec<(String, String)>,
    /// The table rename this change carries or follows.
    pub renames: TableRenames,
    pub added_fks: Vec<ParsedFk>,
    pub dropped_fks: Vec<ParsedFk>,
    pub added_indexes: Vec<ParsedIndex>,
    pub dropped_indexes: Vec<ParsedIndex>,
    /// What happens to the table itself.
    pub table: TableChange,
}

impl Changes {
    /// True if this change set has no effect — a fresh migration would have
    /// nothing to write.
    pub fn is_empty(&self) -> bool {
        self.table == TableChange::Altered
            && self.added_columns.is_empty()
            && self.dropped_columns.is_empty()
            && self.modified_columns.is_empty()
            && self.renamed_columns.is_empty()
            && self.renames.is_empty()
            && self.added_fks.is_empty()
            && self.dropped_fks.is_empty()
            && self.added_indexes.is_empty()
            && self.dropped_indexes.is_empty()
    }
}
