use runique::prelude::*;

model! {
    CourIa,
    table: "cour_ia",
    pk: id => Pk,
    {
        context:       richtext [required],
        contraintes:   richtext [required],
        contrainte_id: int [required],
        cour_id:       int [required],
        sort_order:    int [required],
    }
}
