use crate::entities::cour_block;
use runique::prelude::*;

#[form(schema = cour_block, fields = [chapitre_id, heading, content, block_type, sort_order])]
pub struct CourBlockForm;

#[async_trait]
impl RuniqueForm for CourBlockForm {
    impl_form_access!(model);
}
