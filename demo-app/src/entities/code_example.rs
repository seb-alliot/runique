use runique::prelude::*;

model! {
    CodeExample,
    table: "code_example",
    pk: id => i32,
    fields: {
        page_id: i32 [required],
        title: String [required],
        language: String [required],
        code: String [required],
        context: String [nullable],
        sort_order: i32 [required],
    },
    relations: {
        belongs_to: demo_page via page_id [cascade],
    }
}
