//! `ModelSchema`: single source of truth for a model — columns, primary keys, FKs, indexes, hooks.
use crate::migration::{
    column::ColumnDef, foreign_key::ForeignKeyDef, hooks::HooksDef, index::IndexDef,
    primary_key::PrimaryKeyDef,
};

/// Sort direction for `ModelSchema::order_by`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OrderDir {
    Asc,
    Desc,
}

/// The root struct — single source of truth for the model
#[derive(Debug, Clone)]
pub struct ModelSchema {
    pub model_name: String,
    pub table_name: String,     // snake_case of model_name by default
    pub schema: Option<String>, // PostgreSQL schema (e.g. "public")
    pub primary_key: Option<PrimaryKeyDef>,
    pub columns: Vec<ColumnDef>,
    pub foreign_keys: Vec<ForeignKeyDef>,
    pub indexes: Vec<IndexDef>,
    pub hooks: Option<HooksDef>,
    pub ordering: Vec<(String, OrderDir)>,
    pub unique_together: Vec<Vec<String>>,
    pub verbose_name: Option<String>,
    pub verbose_name_plural: Option<String>,
}

impl ModelSchema {
    /// Creates a schema for `model_name`, deriving its default table name via
    /// PascalCase → snake_case conversion. The primary key must still be set
    /// with [`ModelSchema::primary_key`] before [`ModelSchema::build`] succeeds.
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
            indexes: Vec::new(),
            hooks: None,
            ordering: Vec::new(),
            unique_together: Vec::new(),
            verbose_name: None,
            verbose_name_plural: None,
        }
    }

    // ── Configuration ───────────────────────────────────────────────────────

    /// Overrides the default (auto-derived) table name.
    pub fn table_name(mut self, name: impl Into<String>) -> Self {
        self.table_name = name.into();
        self
    }

    /// Sets the PostgreSQL schema the table lives in (e.g. `"public"`).
    pub fn schema(mut self, schema: impl Into<String>) -> Self {
        self.schema = Some(schema.into());
        self
    }

    // ── Primary key ─────────────────────────────────────────────────────────

    /// Sets the model's primary key. Required — [`ModelSchema::build`] fails without one.
    pub fn primary_key(mut self, pk: PrimaryKeyDef) -> Self {
        self.primary_key = Some(pk);
        self
    }

    // ── Columns ─────────────────────────────────────────────────────────────

    /// Appends a column to the schema.
    pub fn column(mut self, col: ColumnDef) -> Self {
        self.columns.push(col);
        self
    }

    // ── Foreign keys ────────────────────────────────────────────────────────

    /// Appends a foreign key constraint to the schema.
    pub fn foreign_key(mut self, fk: ForeignKeyDef) -> Self {
        self.foreign_keys.push(fk);
        self
    }

    // ── Index ───────────────────────────────────────────────────────────────

    /// Appends an index to the schema.
    pub fn index(mut self, idx: IndexDef) -> Self {
        self.indexes.push(idx);
        self
    }

    // ── Hooks ───────────────────────────────────────────────────────────────

    /// Attaches lifecycle hooks (before/after save/delete) to the model.
    pub fn hooks(mut self, hooks: HooksDef) -> Self {
        self.hooks = Some(hooks);
        self
    }

    // ── Meta ────────────────────────────────────────────────────────────────

    /// Appends a default ordering clause (`field`, direction). Multiple calls
    /// build a multi-column ordering, applied in call order.
    pub fn order_by(mut self, field: impl Into<String>, dir: OrderDir) -> Self {
        self.ordering.push((field.into(), dir));
        self
    }

    /// Adds a unique-together constraint over `fields`.
    pub fn unique_together(mut self, fields: Vec<String>) -> Self {
        self.unique_together.push(fields);
        self
    }

    /// Sets the model's singular display name (used in admin UI).
    pub fn verbose_name(mut self, name: impl Into<String>) -> Self {
        self.verbose_name = Some(name.into());
        self
    }

    /// Sets the model's plural display name (used in admin UI).
    pub fn verbose_name_plural(mut self, name: impl Into<String>) -> Self {
        self.verbose_name_plural = Some(name.into());
        self
    }

    // ── Build ───────────────────────────────────────────────────────────────

    /// Finalizes the schema, failing if no primary key was set.
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

    /// Fills a Forms with fields generated from the schema.
    /// - `fields`: whitelist (only these fields are included, in this order)
    /// - `exclude`: blacklist (these fields are excluded)
    ///
    /// If `fields` is provided, `exclude` is ignored.
    /// The PK is always excluded.
    pub fn fill_form(
        &self,
        form: &mut crate::forms::Forms,
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

    /// Checks, after `customize`, what each column's DSL declaration imposes
    /// on its form field: an integer keeps its Rust type's range, an upload
    /// its size limit, and the declared `min_length`/`max_length`/`min`/`max`
    /// may be tightened, never loosened. Breaking what the column needs is a
    /// programming error, refused the same way as an unknown field in
    /// `fill_form`: a `password` column must stay a password field (hashed,
    /// never sent back), an integer column an integer field.
    pub fn enforce_limits(&self, form: &mut crate::forms::Forms) {
        use runique_dsl::types::Widget;

        for col in self.columns.iter().filter(|c| !c.ignored) {
            let Some(kind) = col.kind else { continue };
            let Some(field) = form.fields.get_mut(&col.name) else {
                continue;
            };
            match kind.widget() {
                Widget::Password if field.field_type() != "password" || !field.is_password() => {
                    panic!(
                        "ModelForm '{}': field '{}' is declared `password`; `customize` replaced it with a `{}` field, which would neither hash nor hide it",
                        self.model_name,
                        col.name,
                        field.field_type()
                    );
                }
                Widget::Integer { min, max } if !field.set_type_bounds(min, max) => {
                    panic!(
                        "ModelForm '{}': field '{}' is declared `{:?}`; `customize` replaced it with a `{}` field, but it must stay an integer field",
                        self.model_name,
                        col.name,
                        kind,
                        field.field_type()
                    );
                }
                _ => {}
            }
            let bounds = field.bounds();
            let loosened = |what: &str, model: String, form: String| -> ! {
                panic!(
                    "ModelForm '{}': field '{}' — `customize` loosened `{what}`: the model declares {model}, the form allows {form}",
                    self.model_name, col.name
                )
            };
            let shown = |v: Option<String>| v.unwrap_or_else(|| "no limit".to_string());
            if let Some(max) = col.max_length
                && !matches!(kind.widget(), Widget::Binary)
                && bounds.max_length.is_none_or(|f| f > max)
            {
                loosened(
                    "max_length",
                    max.to_string(),
                    shown(bounds.max_length.map(|v| v.to_string())),
                );
            }
            if let Some(min) = col.min_length
                && bounds.min_length.is_none_or(|f| f < min)
            {
                loosened(
                    "min_length",
                    min.to_string(),
                    shown(bounds.min_length.map(|v| v.to_string())),
                );
            }
            if let Some(max) = col.max_value
                && bounds.max_int.is_none_or(|f| f > max)
            {
                loosened(
                    "max",
                    max.to_string(),
                    shown(bounds.max_int.map(|v| v.to_string())),
                );
            }
            if let Some(min) = col.min_value
                && bounds.min_int.is_none_or(|f| f < min)
            {
                loosened(
                    "min",
                    min.to_string(),
                    shown(bounds.min_int.map(|v| v.to_string())),
                );
            }
            if let Some(max) = col.max_float
                && bounds.max_float.is_none_or(|f| f > max)
            {
                loosened(
                    "max",
                    max.to_string(),
                    shown(bounds.max_float.map(|v| v.to_string())),
                );
            }
            if let Some(min) = col.min_float
                && bounds.min_float.is_none_or(|f| f < min)
            {
                loosened(
                    "min",
                    min.to_string(),
                    shown(bounds.min_float.map(|v| v.to_string())),
                );
            }
            let size = kind
                .byte_limit(col.max_length)
                .map(u64::from)
                .or(col.max_size);
            if let Some(bytes) = size {
                field.cap_max_size(bytes);
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
    /// Creates an empty diff for `table_name`.
    pub fn new(table_name: &str) -> Self {
        Self {
            table_name: table_name.to_string(),
            added_columns: Vec::new(),
            dropped_columns: Vec::new(),
            modified_columns: Vec::new(),
        }
    }

    /// True if the diff carries no changes at all.
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
