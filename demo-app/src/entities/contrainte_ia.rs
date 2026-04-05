use runique::prelude::*;

model! {
    ContrainteIa,
    table: "contrainte_ia",
    pk: id => Pk,
    fields: {
        contrainte_ia: String [required],
        lang: String [required],
    }
}
