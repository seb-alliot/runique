use runique::prelude::*;

model! {
    EihwazGroupe,
    table: "eihwaz_groupes",
    pk: id => Pk,
    {
        nom: text [max_length: 100, unique, required],
    }
}
