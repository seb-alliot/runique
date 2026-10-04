//! Table index definition — columns, uniqueness, optional name, SeaQuery generation.

/// Index definition.
#[derive(Debug, Clone)]
pub struct IndexDef {
    pub columns: Vec<String>,
    pub unique: bool,
    pub name: Option<String>,
}

impl IndexDef {
    /// Creates an index over `columns`. Not unique and unnamed by default.
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
}
