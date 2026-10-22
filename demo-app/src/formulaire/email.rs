use runique::prelude::*;
use serde::Serialize;

// --- EMAIL FORM ---
#[derive(Serialize, Debug, Clone)]
#[serde(transparent)]
pub struct EmailForm {
    pub form: Forms,
}

impl RuniqueForm for EmailForm {
    fn register_fields(form: &mut Forms) {
        form.field(
            &TextField::email("email")
                .label("Entrez un email")
                .required(),
        );
    }
    impl_form_access!();

    fn allow_post(&self, _request: &Request) -> bool {
        false
    }

    fn allow_get(&self, _request: &Request) -> bool {
        self.is_submitted()
    }
}
