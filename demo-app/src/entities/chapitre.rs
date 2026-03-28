use runique::prelude::*;

model! {
    Chapitre,
    table: "chapitre",
    pk: id => i32,
    fields: {
        cour_id: i32 [required],
        slug: String [required],
        title: String [required],
        lead: String [nullable],
        sort_order: i32 [required],
    },
    relations: {
        belongs_to: Cour via cour_id [cascade],
    }
}
