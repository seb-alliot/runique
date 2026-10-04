//! Foreign key definition — target table, column, ON DELETE / ON UPDATE actions.
//!
//! [`ForeignKeyDef`] follows the builder pattern:
//! `ForeignKeyDef::new("user_id").references("users").on_delete(ForeignKeyAction::Cascade)`.
use sea_query::ForeignKeyAction;

/// Foreign key definition.
#[derive(Debug, Clone)]
pub struct ForeignKeyDef {
    pub from_column: String,
    pub to_table: String,
    pub to_column: String,
    pub on_delete: ForeignKeyAction,
    pub on_update: ForeignKeyAction,
}

impl ForeignKeyDef {
    /// Creates a foreign key from `from_column`, defaulting to `id` on the
    /// referenced table with `NoAction` on both delete and update.
    pub fn new(from_column: impl Into<String>) -> Self {
        Self {
            from_column: from_column.into(),
            to_table: String::new(),
            to_column: "id".to_string(),
            on_delete: ForeignKeyAction::NoAction,
            on_update: ForeignKeyAction::NoAction,
        }
    }

    /// Sets the referenced table.
    pub fn references(mut self, table: impl Into<String>) -> Self {
        self.to_table = table.into();
        self
    }

    /// Sets the referenced column (defaults to `id`).
    pub fn to_column(mut self, column: impl Into<String>) -> Self {
        self.to_column = column.into();
        self
    }

    /// Sets the `ON DELETE` action.
    pub fn on_delete(mut self, action: ForeignKeyAction) -> Self {
        self.on_delete = action;
        self
    }

    /// Sets the `ON UPDATE` action.
    pub fn on_update(mut self, action: ForeignKeyAction) -> Self {
        self.on_update = action;
        self
    }
}
