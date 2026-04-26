use runique::prelude::*;
use serde::Serialize;

// Manual form — fields declared explicitly, not tied to an entity schema.
// Use this approach when your form does not map directly to a database table
// (e.g. login, search, contact forms).
//
// WARNING: Do not declare non-entity models inside src/entities/.
// `runique makemigrations` scans that folder and will attempt to generate
// a migration for any model found there.
#[derive(Serialize, Debug, Clone)]
#[serde(transparent)]
pub struct LoginForm {
    pub form: Forms,
}

impl RuniqueForm for LoginForm {
    fn register_fields(form: &mut Forms) {
        form.field(&TextField::text("username").label("Username").required());
        form.field(&TextField::password("password").label("Password").required());
    }

    impl_form_access!();
}
