use sea_query::ColumnType;

/// Definition of the primary key
#[derive(Debug, Clone)]
pub struct PrimaryKeyDef {
    pub name: String,
    pub col_type: ColumnType,
    pub auto_increment: bool,
}

impl PrimaryKeyDef {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            col_type: ColumnType::Integer,
            auto_increment: true,
        }
    }

    pub fn i32(mut self) -> Self {
        self.col_type = ColumnType::Integer;
        self
    }

    pub fn i64(mut self) -> Self {
        self.col_type = ColumnType::BigInteger;
        self
    }

    pub fn uuid(mut self) -> Self {
        self.col_type = ColumnType::Uuid;
        self.auto_increment = false;
        self
    }

    pub fn auto_increment(mut self) -> Self {
        self.auto_increment = true;
        self
    }

    pub fn no_auto_increment(mut self) -> Self {
        self.auto_increment = false;
        self
    }

    /// Generates the corresponding SeaQuery ColumnDef
    pub fn to_sea_column(&self) -> sea_query::ColumnDef {
        let mut col = sea_query::ColumnDef::new_with_type(
            sea_query::Alias::new(&self.name),
            self.col_type.clone(),
        );
        col.not_null().primary_key();
        if self.auto_increment {
            col.auto_increment();
        }
        col
    }
}
