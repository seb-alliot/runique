use runique::prelude::*;

#[derive(Serialize, Debug, Clone)]
#[serde(transparent)]
pub struct SearchDemoForm {
    pub form: Forms,
}

impl RuniqueForm for SearchDemoForm {
    impl_form_access!();

    fn register_fields(form: &mut Forms) {
        form.field(
            &TextField::text("search")
                .label("Recherche")
                .placeholder("ex: runique"),
        );
        form.field(
            &TextField::text("category")
                .label("Catégorie")
                .placeholder("ex: web"),
        );
    }
}
