use crate::entities::cour::schema as cour;
use runique::prelude::*;

#[form(schema = cour, fields = [slug, lang, title, theme, difficulte, ordre, sort_order])]
pub struct CourForm;

#[async_trait]
impl RuniqueForm for CourForm {
    impl_form_access!(model);
}
