//! One model's admin: one block per responsibility, each set through a closure.
use super::form::FormConfig;
use super::list::ListConfig;
use crate::admin::refonte_admin::meta::ModelMeta;

pub struct ModelAdmin<E: ModelMeta> {
    pub list: ListConfig<E>,
    pub form: FormConfig<E>,
}

impl<E: ModelMeta> Default for ModelAdmin<E> {
    fn default() -> Self {
        Self {
            list: ListConfig::default(),
            form: FormConfig::default(),
        }
    }
}

impl<E: ModelMeta> ModelAdmin<E> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn list(mut self, l: impl FnOnce(ListConfig<E>) -> ListConfig<E>) -> Self {
        self.list = l(self.list);
        self
    }

    pub fn form(mut self, f: impl FnOnce(FormConfig<E>) -> FormConfig<E>) -> Self {
        self.form = f(self.form);
        self
    }

    pub(crate) fn validate(&self) -> Result<(), String> {
        let mut errors = Vec::new();
        if let Err(e) = self.list.validate() {
            errors.push(e);
        }
        if let Err(e) = self.form.validate() {
            errors.push(e);
        }
        if E::IS_ACCOUNT_TABLE && self.form.has_field("is_superuser") {
            errors.push("fields: `is_superuser` can never be written from the admin".to_string());
        }
        if errors.is_empty() {
            Ok(())
        } else {
            Err(format!("admin `{}`:\n{}", E::KEY, errors.join("\n")))
        }
    }
}
