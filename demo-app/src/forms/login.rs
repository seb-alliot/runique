use crate::backend::lang::visitor_lang;
use runique::axum::http::HeaderMap;
use runique::prelude::*;
use serde::Serialize;

#[derive(Serialize, Debug, Clone)]
#[serde(transparent)]
pub struct LoginForm {
    pub form: Forms,
}

impl FormAccess for LoginForm {
    fn register_fields(form: &mut Forms) {
        form.field(&TextField::text("username").label("Username").required());
        form.field(
            &TextField::password("password")
                .label("Password")
                .no_hash()
                .required(),
        );
    }
    impl_form_access!();
}

impl RuniqueForm for LoginForm {}

impl LoginForm {
    pub fn in_language(mut self, headers: &HeaderMap) -> Self {
        if visitor_lang(headers) == "fr" {
            self.forms_mut()
                .field_label("username", "Nom d'utilisateur")
                .field_label("password", "Mot de passe");
        }
        self
    }
}
