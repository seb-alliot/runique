use runique::prelude::*;

model! {
    ContrainteIa,
    table: "contrainte_ia",
    pk: id => i32,
    fields: {
        contrainte_ia: String [required],
        lang: String [required],
    }
}