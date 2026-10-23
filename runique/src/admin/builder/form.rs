use sea_orm::{EntityTrait, IdenStatic, Iterable, PrimaryKeyToColumn};

pub struct FormConfig<E: EntityTrait> {
    fields: Vec<E::Column>,
    readonly: Vec<E::Column>,
    create_only: Vec<E::Column>,
    labels: Vec<(E::Column, String)>,
}

impl<E: EntityTrait> Default for FormConfig<E> {
    fn default() -> Self {
        Self {
            fields: Vec::new(),
            readonly: Vec::new(),
            create_only: Vec::new(),
            labels: Vec::new(),
        }
    }
}

impl<E: EntityTrait> FormConfig<E> {
    pub fn fields(mut self, cols: impl IntoIterator<Item = E::Column>) -> Self {
        self.fields = cols.into_iter().collect();
        self
    }

    pub(super) fn has_field(&self, name: &str) -> bool {
        self.fields.iter().any(|c| c.as_str() == name)
    }

    /// What  a create form writes: in `fields`, not `readonly`.
    pub(super) fn writable_on_create(&self, name: &str) -> bool {
        self.has_field(name) && !self.readonly.iter().any(|c| c.as_str() == name)
    }

    /// what an edit writes: the create ones, minus those frozen after creation.
    pub(super) fn writable_on_edit(&self, name: &str) -> bool {
        self.writable_on_create(name) && !self.create_only.iter().any(|c| c.as_str() == name)
    }

    /// The columns the form shows, in their declared order.
    pub(super) fn field_columns(&self) -> &[E::Column] {
        &self.fields
    }

    /// No fields declared: the model has no form, so no create page.
    pub(super) fn is_empty(&self) -> bool {
        self.fields.is_empty()
    }
    /// The columns a row's page reads: the form's fields (read-only ones
    /// included), plus the primary key. Nothing else leaves the table.
    pub(crate) fn read_columns(&self) -> Vec<E::Column> {
        let mut cols = self.fields.clone();
        for key in E::PrimaryKey::iter() {
            let col = key.into_column();
            if !cols.iter().any(|c| c.as_str() == col.as_str()) {
                cols.push(col);
            }
        }
        cols
    }

    pub fn readonly(mut self, cols: impl IntoIterator<Item = E::Column>) -> Self {
        self.readonly = cols.into_iter().collect();
        self
    }

    /// Typed when the row is createdn then shwon  but frozen on edit.
    pub fn create_only(mut self, cols: impl IntoIterator<Item = E::Column>) -> Self {
        self.create_only = cols.into_iter().collect();
        self
    }

    pub fn labels<S: Into<String>>(
        mut self,
        labels: impl IntoIterator<Item = (E::Column, S)>,
    ) -> Self {
        self.labels = labels
            .into_iter()
            .map(|(col, label)| (col, label.into()))
            .collect();
        self
    }

    /// The lists of this block that name the column `name`.
    pub(super) fn lists_using(&self, name: &str) -> Vec<&'static str> {
        let mut lists = Vec::new();
        if self.has_field(name) {
            lists.push("fields");
        }
        if self.readonly.iter().any(|c| c.as_str() == name) {
            lists.push("readonly");
        }
        if self.create_only.iter().any(|c| c.as_str() == name) {
            lists.push("create_only")
        }
        if self.labels.iter().any(|(c, _)| c.as_str() == name) {
            lists.push("labels");
        }
        lists
    }

    pub(crate) fn validate(&self) -> Result<(), String> {
        let mut form_error = Vec::new();
        for (field, cols) in [
            ("readonly", &self.readonly),
            ("create_only", &self.create_only),
        ] {
            for col in cols {
                if !self.fields.iter().any(|c| c.as_str() == col.as_str()) {
                    form_error.push(format!(
                        "Field '{}' in '{}' is not present in 'fields'",
                        col.as_str(),
                        field
                    ));
                }
            }
        }

        for col in &self.create_only {
            if self.readonly.iter().any(|c| c.as_str() == col.as_str()) {
                form_error.push(format!(
                    "Field `{}` is in both `readonly` and `create_only`",
                    col.as_str()
                ));
            }
        }
        for (col, _) in &self.labels {
            if !self.fields.iter().any(|c| c.as_str() == col.as_str()) {
                form_error.push(format!(
                    "Field '{}' in 'labels' is not present in 'fields'",
                    col.as_str()
                ));
            }
        }
        if !form_error.is_empty() {
            return Err(form_error.join("\n"));
        }
        Ok(())
    }
}

#[cfg(feature = "sqlite")]
#[cfg(test)]
mod tests {
    use super::FormConfig;
    use crate::auth::gate::test_support::fixtures::note::{Column, Entity};
    #[test]
    fn lists_using_names_exactly_the_lists_holding_the_column() {
        let form = FormConfig::<Entity>::default()
            .fields([Column::Id, Column::OwnerId])
            .readonly([Column::OwnerId])
            .create_only([Column::Id])
            .labels([(Column::OwnerId, "Propriétaire")]);
        assert_eq!(
            form.lists_using("owner_id"),
            ["fields", "readonly", "labels"]
        );
        assert_eq!(form.lists_using("id"), ["fields", "create_only"]);
        assert!(form.lists_using("ghost").is_empty());
    }

    #[test]
    fn readonly_and_labels_inside_fields_are_accepted() {
        let form = FormConfig::<Entity>::default()
            .fields([Column::Id, Column::OwnerId])
            .readonly([Column::Id])
            .labels([(Column::OwnerId, "Propriétaire".to_string())]);
        assert!(form.validate().is_ok());
    }

    #[test]
    fn a_readonly_column_outside_fields_is_refused() {
        let form = FormConfig::<Entity>::default()
            .fields([Column::Id])
            .readonly([Column::OwnerId]);
        let error = form.validate().unwrap_err();
        assert!(error.contains("readonly"), "{error}");
        assert!(error.contains("owner_id"), "{error}");
    }

    #[test]
    fn a_label_outside_fields_is_refused() {
        let form = FormConfig::<Entity>::default()
            .fields([Column::Id])
            .labels([(Column::OwnerId, "Propriétaire".to_string())]);
        let error = form.validate().unwrap_err();
        assert!(error.contains("labels"), "{error}");
    }
    #[test]
    fn create_only_inside_fields_is_accepted() {
        let form = FormConfig::<Entity>::default()
            .fields([Column::OwnerId])
            .create_only([Column::OwnerId]);
        assert!(form.validate().is_ok());
    }

    #[test]
    fn create_only_outside_fields_is_refused() {
        let form = FormConfig::<Entity>::default()
            .fields([Column::Id])
            .create_only([Column::OwnerId]);
        let error = form.validate().unwrap_err();
        assert!(error.contains("create_only"), "{error}");
        assert!(error.contains("owner_id"), "{error}");
    }

    #[test]
    fn a_column_both_readonly_and_create_only_is_refused() {
        let form = FormConfig::<Entity>::default()
            .fields([Column::OwnerId])
            .readonly([Column::OwnerId])
            .create_only([Column::OwnerId]);
        let error = form.validate().unwrap_err();
        assert!(error.contains("both"), "{error}");
    }

    /// The whole table of what each list writes, create and edit side by side.
    #[test]
    fn each_list_is_written_on_create_and_edit_as_declared() {
        let plain = FormConfig::<Entity>::default().fields([Column::OwnerId]);
        assert!(plain.writable_on_create("owner_id"));
        assert!(plain.writable_on_edit("owner_id"));

        let frozen = FormConfig::<Entity>::default()
            .fields([Column::OwnerId])
            .create_only([Column::OwnerId]);
        assert!(frozen.writable_on_create("owner_id"));
        assert!(!frozen.writable_on_edit("owner_id"));

        let shown = FormConfig::<Entity>::default()
            .fields([Column::OwnerId])
            .readonly([Column::OwnerId]);
        assert!(!shown.writable_on_create("owner_id"));
        assert!(!shown.writable_on_edit("owner_id"));
    }
}
