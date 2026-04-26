use crate::entities::doc_block;
use runique::prelude::*;

#[form(schema = doc_block, fields = [page_id, heading, content, block_type, sort_order])]
pub struct DocBlockForm;

#[async_trait]
impl RuniqueForm for DocBlockForm {
    impl_form_access!(model);
}
