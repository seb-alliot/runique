use runique::prelude::*;

model! {
    DocBlock,
    table: "doc_block",
    pk: id => Pk,
    enums: {
        BlockType: [Text, Code, Sommaire],
    },
    {
        page_id:    int [required],
        heading:    text,
        content:    richtext [rows: 12, required],
        block_type: choice [enum(BlockType), required],
        sort_order: int [required],
    },
    relations: {
        belongs_to: doc_page via page_id [cascade],
    }
}
