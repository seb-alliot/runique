use runique::prelude::*;

model! {
    CourIa,
    table: "cour_ia",
    pk: id => Pk,
    {
        context:       richtext [required],
        contraintes:   richtext [required],
        contrainte_id: Pk [required],
        cour_id:       Pk [required],
        sort_order:    int [required],
    }
}
