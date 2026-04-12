use runique::prelude::*;

model! {
    CourBlock,
    table: "cour_block",
    pk: id => Pk,
    {
        chapitre_id: int [required],
        heading:     text,
        content:     richtext [required],
        block_type:  text [required],
        sort_order:  int [required],
    },
    relations: {
        belongs_to: Chapitre via chapitre_id [cascade],
    }
}
