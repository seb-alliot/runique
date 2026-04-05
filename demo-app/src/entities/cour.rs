use runique::prelude::*;

model! {
    Cour,
    table: "cour",
    pk: id => Pk,
    fields: {
        slug: String [required],
        lang: String [required],
        title: String [required],
        theme: String [required],
        difficulte: String [required],
        ordre: i32 [required, label("Ordre")],
        sort_order: i32 [required,  label("Ordre d'affichage")],
    }
}
