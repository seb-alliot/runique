use runique::prelude::*;
use serde::Serialize;

// A manual form: its fields are declared here, not taken from a model. Use one
// when the form doesn't map to a database table (sign in, search, contact).
//
// Don't put such forms in src/entities/: `runique makemigrations` scans that
// folder and generates a migration for every model it finds there.
/// The sign-in form: `username` and `password`, declared by hand.
#[derive(Serialize, Debug, Clone)]
#[serde(transparent)]
pub struct LoginForm {
    pub form: Forms,
}

impl FormAccess for LoginForm {
    fn register_fields(form: &mut Forms) {
        form.field(&TextField::text("username").label("Username").required());
        form.field(&TextField::password("password").label("Password").required());
    }
    impl_form_access!();
}

impl RuniqueForm for LoginForm {}
