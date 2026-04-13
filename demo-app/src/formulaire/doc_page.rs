use crate::entities::doc_page;
use runique::prelude::*;

#[form(schema = doc_page, fields = [section_id, slug, lang, title, lead, sort_order])]
pub struct DocPageForm;

#[async_trait]
impl RuniqueForm for DocPageForm {
    impl_form_access!(model);
}
