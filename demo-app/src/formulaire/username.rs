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

    // `/view-user` is registered for every method (`view!{}`), but this form
    // is a GET-only search — never attempt validation on POST/PUT/PATCH/DELETE,
    // matching the handler's previous explicit `request.is_get() && ...` check.
    fn allow_post(&self, _request: &Request) -> bool {
        false
    }

    // Opt-in: a read-only search form is the one case where auto-validating
    // on GET is safe and desired (`allow_get` defaults to `false`).
    fn allow_get(&self, _request: &Request) -> bool {
        self.is_submitted()
    }
}
