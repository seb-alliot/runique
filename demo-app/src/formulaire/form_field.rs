use crate::entities::form_field::schema as form_field;
use runique::prelude::*;

#[form(schema = form_field, fields = [page_id, name, field_type, description, example, sort_order])]
pub struct FormFieldForm;

#[async_trait]
impl RuniqueForm for FormFieldForm {
    impl_form_access!(model);
}
