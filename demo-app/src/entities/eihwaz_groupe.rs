use runique::prelude::*;

model! {
    EihwazGroupe,
    table: "eihwaz_groupes",
    pk: id => i32,
    fields: {
        nom: String [required, max_len(100), unique],
    }
}
