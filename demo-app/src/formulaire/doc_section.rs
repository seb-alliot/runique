use crate::entities::doc_section::schema as doc_section;
use runique::prelude::*;

#[form(schema = doc_section, fields = [slug, lang, title, sort_order])]
pub struct DocSectionForm;

#[async_trait]
impl RuniqueForm for DocSectionForm {
    impl_form_access!(model);
}
