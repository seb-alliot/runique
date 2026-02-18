use sea_query::ForeignKeyAction;

/// Definition of a foreign key
#[derive(Debug, Clone)]
pub struct ForeignKeyDef {
    pub from_column: String,
    pub to_table: String,
    pub to_column: String,
    pub on_delete: ForeignKeyAction,
    pub on_update: ForeignKeyAction,
}

impl ForeignKeyDef {
    pub fn new(from_column: impl Into<String>) -> Self {
        Self {
            from_column: from_column.into(),
            to_table: String::new(),
            to_column: "id".to_string(),
            on_delete: ForeignKeyAction::NoAction,
            on_update: ForeignKeyAction::NoAction,
        }
    }

    pub fn references(mut self, table: impl Into<String>) -> Self {
        self.to_table = table.into();
        self
    }

    pub fn to_column(mut self, column: impl Into<String>) -> Self {
        self.to_column = column.into();
        self
    }

    pub fn on_delete(mut self, action: ForeignKeyAction) -> Self {
        self.on_delete = action;
        self
    }

    pub fn on_update(mut self, action: ForeignKeyAction) -> Self {
        self.on_update = action;
        self
    }

    /// Generates the corresponding SeaQuery ForeignKey
    pub fn to_sea_foreign_key(&self, from_table: &str) -> sea_query::ForeignKeyCreateStatement {
        sea_query::ForeignKey::create()
            .from(
                sea_query::Alias::new(from_table),
                sea_query::Alias::new(&self.from_column),
            )
            .to(
                sea_query::Alias::new(&self.to_table),
                sea_query::Alias::new(&self.to_column),
            )
            .on_delete(self.on_delete)
            .on_update(self.on_update)
            .to_owned()
    }
}
