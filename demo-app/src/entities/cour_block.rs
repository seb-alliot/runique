use runique::prelude::*;

model! {
    CourBlock,
    table: "cour_block",
    pk: id => i32,
    fields: {
        chapitre_id: i32 [required],
        heading: String [nullable],
        content: String [required],
        block_type: String [required],
        sort_order: i32 [required],
    },
    relations: {
        belongs_to: Chapitre via chapitre_id [cascade],
    }
}
