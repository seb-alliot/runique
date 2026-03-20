use crate::entities::demo_section::schema as demo_section;
use runique::prelude::*;

#[form(schema = demo_section, fields = [page_id, title, content, sort_order])]
pub struct DemoSectionForm;

#[async_trait]
impl RuniqueForm for DemoSectionForm {
    impl_form_access!(model);
}
