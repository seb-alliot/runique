use runique::prelude::*;

model! {
    EihwazDroit,
    table: "eihwaz_droits",
    pk: id => i32,
    fields: {
        nom: String [required, max_len(100), unique],
    }
}

