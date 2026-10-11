use sea_orm::{EntityTrait, IdenStatic};

pub struct FormConfig<E: EntityTrait> {
    fields: Vec<E::Column>,
    readonly: Vec<E::Column>,
    labels: Vec<(E::Column, String)>,
}

impl<E: EntityTrait> Default for FormConfig<E> {
    fn default() -> Self {
        Self {
            fields: Vec::new(),
            readonly: Vec::new(),
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

    pub fn readonly(mut self, cols: impl IntoIterator<Item = E::Column>) -> Self {
        self.readonly = cols.into_iter().collect();
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

    pub(crate) fn validate(&self) -> Result<(), String> {
        let mut form_error = Vec::new();
        for (field, cols) in [("readonly", &self.readonly)] {
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
    use crate::admin::refonte_admin::gate::test_support::fixtures::note::{Column, Entity};

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
}
