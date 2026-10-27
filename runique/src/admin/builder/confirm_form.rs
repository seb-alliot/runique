//! The form a confirmation sends (delete a row, reset a password): no
//! field, only its CSRF token and its name. Runique's own: the model's form
//! can't serve, its required fields would fail a bare confirm button.
use crate::auth::meta::ModelMeta;
use crate::forms::FormEntity;
use crate::forms::field::{FormAccess, RuniqueForm};
use crate::forms::form::Forms;
use std::marker::PhantomData;

pub(crate) struct ConfirmForm<E> {
    form: Forms,
    entity: PhantomData<fn() -> E>,
}

impl<E: ModelMeta> FormAccess for ConfirmForm<E> {
    fn register_fields(_form: &mut Forms) {}
    fn from_form(form: Forms) -> Self {
        Self {
            form,
            entity: PhantomData,
        }
    }
    fn get_form(&self) -> &Forms {
        &self.form
    }
    fn get_form_mut(&mut self) -> &mut Forms {
        &mut self.form
    }
}

impl<E: ModelMeta> RuniqueForm for ConfirmForm<E> {}

impl<E: ModelMeta> FormEntity for ConfirmForm<E> {
    type Entity = E;
}
