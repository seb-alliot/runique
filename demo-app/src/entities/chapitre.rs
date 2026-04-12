use runique::prelude::*;

model! {
    Chapitre,
    table: "chapitre",
    pk: id => Pk,
    {
        cour_id:    int [required],
        slug:       text [required],
        title:      text [required],
        lead:       text,
        sort_order: int [required],
    },
    relations: {
        belongs_to: Cour via cour_id [cascade],
    }
}
