use runique::prelude::*;

model! {
    DocBlock,
    table: "doc_block",
    pk: id => i32,
    fields: {
        page_id: i32 [required],
        heading: String [nullable],
        content: String [required],
        block_type: String [required],
        sort_order: i32 [required],
    },
    relations: {
        belongs_to: DocPage via page_id [cascade],
    }
}
