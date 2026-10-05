use runique::prelude::*;

model! {
    FormField,
    table: "form_field",
    pk: id => Pk,
    {
        page_id:      int [required],
        name:         text [required],
        field_type:   text [required],
        description:  text [required],
        example:      text [nullable],
        html_preview: text [nullable],
        sort_order:   int [required],
    }
}
