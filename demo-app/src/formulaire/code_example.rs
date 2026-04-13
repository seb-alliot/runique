use crate::entities::code_example;
use runique::prelude::*;

#[form(schema = code_example, fields = [page_id, title, language, code, context, sort_order])]
pub struct CodeExampleForm;

#[async_trait]
impl RuniqueForm for CodeExampleForm {
    impl_form_access!(model);
}
