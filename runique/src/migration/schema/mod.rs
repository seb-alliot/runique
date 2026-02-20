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
                "ModelSchema '{}' : missing primary key",
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

    /// Fills a Forms with fields generated from the schema.
    /// - `fields`: whitelist (only these fields are included, in this order)
    /// - `exclude`: blacklist (these fields are excluded)
    ///
    /// If `fields` is provided, `exclude` is ignored.
    /// The PK is always excluded.
    pub fn fill_form(
        &self,
        form: &mut crate::forms::manager::Forms,
        fields: Option<&[&str]>,
        exclude: Option<&[&str]>,
    ) {
        // Columns always auto-excluded: PK
        let pk_name = self.primary_key.as_ref().map(|pk| pk.name.as_str());

        if let Some(field_names) = fields {
            // Whitelist: respect the order given by the developer
            for &field_name in field_names {
                let col = self.columns.iter().find(|c| c.name == field_name);
                match col {
                    None => panic!(
                        "ModelForm '{}' : field '{}' does not exist in the schema",
                        self.model_name, field_name
                    ),
                    Some(col) => {
                        if let Some(generic) = col.to_form_field() {
                            form.field_generic(generic);
                        }
                    }
                }
            }
        } else {
            // No whitelist: all fields except excluded
            let excluded: &[&str] = exclude.unwrap_or(&[]);

            for col in &self.columns {
                // Skip PK
                if pk_name == Some(col.name.as_str()) {
                    continue;
                }
                // Skip excluded
                if excluded.contains(&col.name.as_str()) {
                    continue;
                }
                if let Some(generic) = col.to_form_field() {
                    form.field_generic(generic);
                }
            }
        }
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

    fn col_to_rust_type(col: &ColumnDef) -> String {
        use sea_query::ColumnType::*;
        let base = match &col.col_type {
            String(_) | Text | Char(_) => "String".to_string(),
            Integer | TinyInteger | SmallInteger => "i32".to_string(),
            BigInteger => "i64".to_string(),
            Unsigned => "u32".to_string(),
            BigUnsigned => "u64".to_string(),
            Float => "f32".to_string(),
            Double => "f64".to_string(),
            Boolean => "bool".to_string(),
            Date => "chrono::NaiveDate".to_string(),
            Time => "chrono::NaiveTime".to_string(),
            DateTime | Timestamp | TimestampWithTimeZone => "chrono::NaiveDateTime".to_string(),
            Uuid => "Uuid".to_string(),
            Json | JsonBinary => "serde_json::Value".to_string(),
            Decimal(_) => "rust_decimal::Decimal".to_string(),
            Enum { .. } => "String".to_string(),
            _ => "String".to_string(),
        };

        if col.nullable {
            format!("Option<{}>", base)
        } else {
            base
        }
    }

    pub fn to_model(&self) -> String {
        let mut out = String::new();
        let table_name = &self.table_name;

        // Imports
        out.push_str("use sea_orm::entity::prelude::*;\n");
        out.push_str("use serde::{Serialize, Deserialize};\n");
        out.push_str("use runique::impl_objects;\n\n");

        // Struct Model
        out.push_str(
            "#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]\n",
        );
        out.push_str(&format!("#[sea_orm(table_name = \"{}\")]\n", table_name));
        out.push_str("pub struct Model {\n");

        // Primary key
        if let Some(ref pk) = self.primary_key {
            out.push_str("    #[sea_orm(primary_key)]\n");
            out.push_str(&format!(
                "    pub {}: {},\n",
                pk.name,
                Self::pk_to_rust_type(pk)
            ));
        }

        // Columns
        for col in &self.columns {
            if col.ignored {
                continue;
            }
            let rust_type = Self::col_to_rust_type(col);
            out.push_str(&format!("    pub {}: {},\n", col.name, rust_type));
        }

        out.push_str("}\n\n");

        // Relation
        out.push_str("#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]\n");
        out.push_str("pub enum Relation {}\n\n");

        // ActiveModelBehavior
        out.push_str("impl ActiveModelBehavior for ActiveModel {}\n\n");

        // impl_objects
        out.push_str("impl_objects!(Entity);\n");

        out
    }
    fn pk_to_rust_type(pk: &PrimaryKeyDef) -> &'static str {
        use sea_query::ColumnType::*;
        match &pk.col_type {
            Integer => "i32",
            BigInteger => "i64",
            Uuid => "Uuid",
            _ => "i32",
        }
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
