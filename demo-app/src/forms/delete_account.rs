use crate::backend::lang::visitor_lang;
use runique::axum::http::HeaderMap;
use runique::prelude::*;
use serde::Serialize;

/// The account deletion confirmation: the current password, read as typed
/// (never hashed) to be checked against the stored one.
#[derive(Serialize, Debug, Clone)]
#[serde(transparent)]
pub struct DeleteAccountForm {
    pub form: Forms,
}

impl FormAccess for DeleteAccountForm {
    fn register_fields(form: &mut Forms) {
        form.field(
            &TextField::password("password")
                .label("Password")
                .no_hash()
                .required(),
        );
    }
    impl_form_access!();
}

impl RuniqueForm for DeleteAccountForm {}

impl DeleteAccountForm {
    pub fn in_language(mut self, headers: &HeaderMap) -> Self {
        if visitor_lang(headers) == "fr" {
            self.forms_mut().field_label("password", "Mot de passe");
        }
        self
    }
}
