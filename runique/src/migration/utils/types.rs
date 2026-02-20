#[derive(Debug, Clone)]
pub struct ParsedSchema {
    pub table_name: String,
    pub primary_key: Option<ParsedColumn>,
    pub columns: Vec<ParsedColumn>,
    pub foreign_keys: Vec<ParsedFk>,
    pub indexes: Vec<ParsedIndex>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ParsedColumn {
    pub name: String,
    pub col_type: String,
    pub nullable: bool,
    pub unique: bool,
    pub ignored: bool,
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

#[derive(Debug)]
pub struct Changes {
    pub table_name: String,
    pub added_columns: Vec<ParsedColumn>,
    pub dropped_columns: Vec<String>,
    pub modified_columns: Vec<(ParsedColumn, ParsedColumn)>,
    pub added_fks: Vec<ParsedFk>,
    pub dropped_fks: Vec<ParsedFk>,
    pub added_indexes: Vec<ParsedIndex>,
    pub dropped_indexes: Vec<ParsedIndex>,
    pub is_new_table: bool,
}

impl Changes {
    /// Vérifie si le diff de schéma est vide (aucune modification).
    ///
    /// # Exemple
    ///
    /// ```rust
    /// use runique::migration::utils::types::Changes;
    /// let changes = Changes {
    ///     table_name: "test".to_string(),
    ///     added_columns: vec![],
    ///     dropped_columns: vec![],
    ///     modified_columns: vec![],
    ///     added_fks: vec![],
    ///     dropped_fks: vec![],
    ///     added_indexes: vec![],
    ///     dropped_indexes: vec![],
    ///     is_new_table: false,
    /// };
    /// assert!(changes.is_empty());
    /// ```
    pub fn is_empty(&self) -> bool {
        !self.is_new_table
            && self.added_columns.is_empty()
            && self.dropped_columns.is_empty()
            && self.modified_columns.is_empty()
            && self.added_fks.is_empty()
            && self.dropped_fks.is_empty()
            && self.added_indexes.is_empty()
            && self.dropped_indexes.is_empty()
    }
}
