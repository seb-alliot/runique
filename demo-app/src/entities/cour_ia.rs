use runique::prelude::*;

model! {
    CourIa,
    table: "cour_ia",
    pk: id => i32,
    fields: {
        context: String [required],
        contraintes: String [required],
        contrainte_id: i32 [required],
        cour_id: i32 [required],
        sort_order: i32 [required],
    }
    relations: {
        belongs_to: ContrainteIa via contrainte_id [cascade],
        belongs_to: Cour via cour_id [cascade],
    }
}
