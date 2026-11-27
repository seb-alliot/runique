use crate::backend::lang::visitor_lang;
use runique::axum::http::HeaderMap;
use runique::prelude::*;

#[derive(Serialize, Debug, Clone)]
#[serde(transparent)]
pub struct SearchDemoForm {
    pub form: Forms,
}

impl FormAccess for SearchDemoForm {
    impl_form_access!();
    fn register_fields(form: &mut Forms) {
        form.field(
            &TextField::text("search")
                .label("Search")
                .placeholder("e.g. runique"),
        );
        form.field(
            &TextField::text("category")
                .label("Category")
                .placeholder("e.g. web"),
        );
    }
}

impl RuniqueForm for SearchDemoForm {}

impl SearchDemoForm {
    pub fn in_language(mut self, headers: &HeaderMap) -> Self {
        if visitor_lang(headers) == "fr" {
            self.forms_mut()
                .field_label("search", "Recherche")
                .field_placeholder("search", "ex : runique")
                .field_label("category", "Catégorie")
                .field_placeholder("category", "ex : web");
        }
        self
    }
}
