use runique::prelude::*;

model! {
    DocBlock,
    table: "doc_block",
    pk: id => Pk,
    enums: {
        BlockType: pg [Text, Code, Sommaire],
    },
    fields: {
        page_id: i32 [required],
        heading: String [nullable],
        content: String [required],
        block_type: enum(BlockType) [required],
        sort_order: i32 [required],
    },
    relations: {
        belongs_to: doc_page via page_id [cascade],
    }
}
