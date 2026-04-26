use crate::entities::page_doc_link;
use runique::prelude::*;

#[form(schema = page_doc_link, fields = [page_id, label, url, link_type, sort_order])]
pub struct PageDocLinkForm;

#[async_trait]
impl RuniqueForm for PageDocLinkForm {
    impl_form_access!(model);
}
