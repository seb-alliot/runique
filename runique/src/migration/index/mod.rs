/// Definition of an index
#[derive(Debug, Clone)]
pub struct IndexDef {
    pub columns: Vec<String>,
    pub unique: bool,
    pub name: Option<String>,
}

impl IndexDef {
    pub fn new(columns: Vec<impl Into<String>>) -> Self {
        Self {
            columns: columns.into_iter().map(|c| c.into()).collect(),
            unique: false,
            name: None,
        }
    }

    pub fn unique(mut self) -> Self {
        self.unique = true;
        self
    }

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
