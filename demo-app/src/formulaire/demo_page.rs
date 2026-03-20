use crate::entities::demo_page::schema as demo_page;
use runique::prelude::*;

#[form(schema = demo_page, fields = [category_id, slug, title, lead, page_type, sort_order])]
pub struct DemoPageForm;

#[async_trait]
impl RuniqueForm for DemoPageForm {
    impl_form_access!(model);
}
