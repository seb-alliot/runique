//! The form a grouped action sends: the selected rows' keys and the action,
//! with its CSRF token and its name. Runique's own, for the list's bar.
use crate::auth::meta::ModelMeta;
use crate::forms::FormEntity;
use crate::forms::field::{FormAccess, RuniqueForm};
use crate::forms::fields::HiddenField;
use crate::forms::form::Forms;
use std::marker::PhantomData;

/// The selected rows' keys, comma-separated.
pub(crate) const BULK_IDS: &str = "ids";
/// `delete`, or `a<n>`: the list's `n`-th declared action.
pub(crate) const BULK_ACTION: &str = "bulk_action";

pub(crate) struct BulkForm<E> {
    form: Forms,
    entity: PhantomData<fn() -> E>,
}

impl<E: ModelMeta> FormAccess for BulkForm<E> {
    fn register_fields(form: &mut Forms) {
        form.field(&HiddenField::new(BULK_IDS));
        form.field(&HiddenField::new(BULK_ACTION));
    }
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

impl<E: ModelMeta> RuniqueForm for BulkForm<E> {}

impl<E: ModelMeta> FormEntity for BulkForm<E> {
    type Entity = E;
}
