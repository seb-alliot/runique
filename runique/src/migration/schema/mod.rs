use crate::migration::{
    column::ColumnDef, foreign_key::ForeignKeyDef, hooks::HooksDef, index::IndexDef,
    primary_key::PrimaryKeyDef, relation::RelationDef,
};

/// The root struct — single source of truth for the model
#[derive(Debug, Clone)]
pub struct ModelSchema {
    pub model_name: String,
    pub table_name: String,     // snake_case of model_name by default
    pub schema: Option<String>, // PostgreSQL schema (e.g. "public")
    pub primary_key: Option<PrimaryKeyDef>,
    pub columns: Vec<ColumnDef>,
    pub foreign_keys: Vec<ForeignKeyDef>,
    pub relations: Vec<RelationDef>,
    pub indexes: Vec<IndexDef>,
    pub hooks: Option<HooksDef>,
}

impl ModelSchema {
    pub fn new(model_name: impl Into<String>) -> Self {
        let name: String = model_name.into();
        // PascalCase → snake_case conversion for table name
        let table_name = to_snake_case(&name);
        Self {
            model_name: name,
            table_name,
            schema: None,
            primary_key: None,
            columns: Vec::new(),
            foreign_keys: Vec::new(),
            relations: Vec::new(),
            indexes: Vec::new(),
            hooks: None,
        }
    }

    // ── Configuration ───────────────────────────────────────────────────────

    pub fn table_name(mut self, name: impl Into<String>) -> Self {
        self.table_name = name.into();
        self
    }

    pub fn schema(mut self, schema: impl Into<String>) -> Self {
        self.schema = Some(schema.into());
        self
    }

    // ── Primary key ─────────────────────────────────────────────────────────

    pub fn primary_key(mut self, pk: PrimaryKeyDef) -> Self {
        self.primary_key = Some(pk);
        self
    }

    // ── Columns ─────────────────────────────────────────────────────────────

    pub fn column(mut self, col: ColumnDef) -> Self {
        self.columns.push(col);
        self
    }

    // ── Foreign keys ────────────────────────────────────────────────────────

    pub fn foreign_key(mut self, fk: ForeignKeyDef) -> Self {
        self.foreign_keys.push(fk);
        self
    }

    // ── Relations ───────────────────────────────────────────────────────────

    pub fn relation(mut self, rel: RelationDef) -> Self {
        self.relations.push(rel);
        self
    }

    // ── Index ───────────────────────────────────────────────────────────────

    pub fn index(mut self, idx: IndexDef) -> Self {
        self.indexes.push(idx);
        self
    }

    // ── Hooks ───────────────────────────────────────────────────────────────

    pub fn hooks(mut self, hooks: HooksDef) -> Self {
        self.hooks = Some(hooks);
        self
    }

    // ── Build ───────────────────────────────────────────────────────────────

    pub fn build(self) -> Result<ModelSchema, String> {
        if self.primary_key.is_none() {
            return Err(format!(
                "ModelSchema '{}' : clé primaire manquante",
                self.model_name
            ));
        }
        Ok(self)
    }

    // ── Migration generation ─────────────────────────────────────────────────

    /// Generates the SeaQuery TableCreateStatement from the schema
    /// This replaces the syn parser — the source of truth is here
    pub fn to_migration(&self) -> sea_query::TableCreateStatement {
        let mut table = sea_query::Table::create();
        table
            .table(sea_query::Alias::new(&self.table_name))
            .if_not_exists();

        // Primary key
        if let Some(ref pk) = self.primary_key {
            table.col(pk.to_sea_column());
        }

        // Columns (ignored fields are skipped)
        for col in &self.columns {
            if !col.ignored {
                table.col(col.to_sea_column());
            }
        }

        // Foreign keys
        for fk in &self.foreign_keys {
            table.foreign_key(&mut fk.to_sea_foreign_key(&self.table_name));
        }

        table.to_owned()
    }

    /// Diff between two ModelSchema — returns the changes to apply
    pub fn diff(&self, other: &ModelSchema) -> SchemaDiff {
        let mut diff = SchemaDiff::new(&self.table_name);

        // Added columns
        let self_cols: std::collections::HashSet<&str> =
            self.columns.iter().map(|c| c.name.as_str()).collect();
        let other_cols: std::collections::HashSet<&str> =
            other.columns.iter().map(|c| c.name.as_str()).collect();

        for name in other_cols.difference(&self_cols) {
            let col = other.columns.iter().find(|c| c.name == *name).unwrap();
            diff.added_columns.push(col.clone());
        }

        for name in self_cols.difference(&other_cols) {
            diff.dropped_columns.push(name.to_string());
        }

        diff
    }
}

/// Result of the diff between two ModelSchema
#[derive(Debug)]
pub struct SchemaDiff {
    pub table_name: String,
    pub added_columns: Vec<ColumnDef>,
    pub dropped_columns: Vec<String>,
    pub modified_columns: Vec<(ColumnDef, ColumnDef)>, // (before, after)
}

impl SchemaDiff {
    pub fn new(table_name: &str) -> Self {
        Self {
            table_name: table_name.to_string(),
            added_columns: Vec::new(),
            dropped_columns: Vec::new(),
            modified_columns: Vec::new(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.added_columns.is_empty()
            && self.dropped_columns.is_empty()
            && self.modified_columns.is_empty()
    }
}

/// PascalCase → snake_case
fn to_snake_case(s: &str) -> String {
    let mut result = String::new();
    for (i, ch) in s.chars().enumerate() {
        if ch.is_uppercase() && i > 0 {
            result.push('_');
        }
        result.push(ch.to_lowercase().next().unwrap());
    }
    result
}
