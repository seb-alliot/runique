use crate::entities::demo_category;
use runique::prelude::*;

#[form(schema = demo_category, fields = [title, back_link_url, back_link_label, sort_order])]
pub struct DemoCategoryForm;

#[async_trait]
impl RuniqueForm for DemoCategoryForm {
    impl_form_access!(model);
}
