use runique::prelude::*;

model! {
    CourIa,
    table: "cour_ia",
    pk: id => Pk,
    fields: {
        context: String [required],
        contraintes: String [required],
        contrainte_id: i32 [required],
        cour_id: i32 [required],
        sort_order: i32 [required],
    }
}
