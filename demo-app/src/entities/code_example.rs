use runique::prelude::*;

model! {
    CodeExample,
    table: "code_example",
    pk: id => Pk,
    {
        page_id:    int [required],
        title:      text [required],
        language:   text [required],
        code:       richtext [required],
        context:    text,
        sort_order: int [required],
    },
    relations: {
        belongs_to: demo_page via page_id [cascade],
    }
}
