use runique::prelude::*;

model! {
    ContrainteIa,
    table: "contrainte_ia",
    pk: id => Pk,
    {
        contrainte_ia: text [required],
        lang:          text [required],
    }
}
