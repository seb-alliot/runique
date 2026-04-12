use runique::prelude::*;

model! {
    EihwazDroit,
    table: "eihwaz_droits",
    pk: id => Pk,
    {
        nom:          text [max_length: 100, unique, required],
        resource_key: text,
        access_type:  text,
    }
}
