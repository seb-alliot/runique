//! Table index definition — columns, uniqueness, optional name, SeaQuery generation.

/// Index definition.
#[derive(Debug, Clone)]
pub struct IndexDef {
    pub columns: Vec<String>,
    pub unique: bool,
    pub name: Option<String>,
}

impl IndexDef {
    /// Creates an index over `columns`. Not unique and unnamed by default —
    /// an unnamed index gets an auto-generated `idx_<table>_<columns>` name
    /// from [`IndexDef::to_sea_index`].
    pub fn new(columns: Vec<impl Into<String>>) -> Self {
        Self {
            columns: columns.into_iter().map(|c| c.into()).collect(),
            unique: false,
            name: None,
        }
    }

    /// Marks the index as unique.
    pub fn unique(mut self) -> Self {
        self.unique = true;
        self
    }

    /// Sets an explicit index name, overriding the auto-generated one.
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Generates the corresponding SeaQuery Index
    pub fn to_sea_index(&self, table: &str) -> sea_query::IndexCreateStatement {
        let index_name = self
            .name
            .clone()
            .unwrap_or_else(|| format!("idx_{}_{}", table, self.columns.join("_")));

        let mut idx = sea_query::Index::create();
        idx.name(&index_name).table(sea_query::Alias::new(table));

        for col in &self.columns {
            idx.col(sea_query::Alias::new(col));
        }

        if self.unique {
            idx.unique();
        }

        idx.to_owned()
    }
}
