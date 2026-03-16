use runique::prelude::*;
use serde::Serialize;

// --- USERNAME FORM ---
#[derive(Serialize, Debug, Clone)]
#[serde(transparent)]
pub struct UsernameForm {
    pub form: Forms,
}

impl RuniqueForm for UsernameForm {
    fn register_fields(form: &mut Forms) {
        form.field(&TextField::text("username").label("Entrez un pseudo"));
    }
    impl_form_access!();
}
