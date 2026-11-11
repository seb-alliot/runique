//! Primary key definition — type (i32, i64, UUID), auto-increment, and SeaQuery generation.
use sea_query::ColumnType;

/// Table primary key definition.
#[derive(Debug, Clone)]
pub struct PrimaryKeyDef {
    pub name: String,
    pub col_type: ColumnType,
    pub auto_increment: bool,
}

impl PrimaryKeyDef {
    /// Creates a primary key named `name`, defaulting to an auto-incrementing `i32`.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            col_type: ColumnType::Integer,
            auto_increment: true,
        }
    }

    /// Uses a 32-bit integer primary key (the default).
    pub fn i32(mut self) -> Self {
        self.col_type = ColumnType::Integer;
        self
    }

    /// Uses a 64-bit integer primary key.
    pub fn i64(mut self) -> Self {
        self.col_type = ColumnType::BigInteger;
        self
    }

    /// Uses a UUID primary key. Disables auto-increment, since UUIDs are
    /// generated client-side rather than by the database sequence.
    pub fn uuid(mut self) -> Self {
        self.col_type = ColumnType::Uuid;
        self.auto_increment = false;
        self
    }

    /// Marks the primary key as auto-incrementing.
    pub fn auto_increment(mut self) -> Self {
        self.auto_increment = true;
        self
    }

    /// Disables auto-increment — the caller is responsible for supplying the
    /// primary key value on insert.
    pub fn no_auto_increment(mut self) -> Self {
        self.auto_increment = false;
        self
    }
}
