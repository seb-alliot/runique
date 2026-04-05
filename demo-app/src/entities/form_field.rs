use runique::prelude::*;

model! {
    FormField,
    table: "form_field",
    pk: id => Pk,
    fields: {
        page_id: i32 [required],
        name: String [required],
        field_type: String [required],
        description: String [required],
        example: String [nullable],
        html_preview: String [nullable],
        sort_order: i32 [required],
    }
}
