use runique::prelude::*;

model! {
    PageDocLink,
    table: "page_doc_link",
    pk: id => i32,
    fields: {
        page_id: i32 [required],
        label: String [required],
        url: String [required],
        link_type: String [required],
        sort_order: i32 [required],
    },
    relations: {
        belongs_to: DemoPage via page_id [cascade],
    }
}
