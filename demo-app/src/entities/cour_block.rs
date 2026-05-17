use runique::prelude::*;

model! {
    CourBlock,
    table: "cour_block",
    pk: id => Pk,
    enums: {
        CourBlockType: [Code = "code", Text = "text"],
    },
    {
        chapitre_id: int [required],
        heading:     text,
        content:     richtext [required],
        block_type:  choice [enum(CourBlockType), required],
        sort_order:  int [required],
    },
    relations: {
        belongs_to: Chapitre via chapitre_id [cascade],
    }
}
