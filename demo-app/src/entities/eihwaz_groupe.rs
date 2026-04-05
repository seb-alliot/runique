use runique::prelude::*;

model! {
    EihwazGroupe,
    table: "eihwaz_groupes",
    pk: id => Pk,
    fields: {
        nom: String [required, max_len(100), unique],
    }
}
