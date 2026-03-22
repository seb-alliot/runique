use runique::prelude::*;

model! {
    Cour,
    table: "cour",
    pk: id => i32,
    fields: {
        slug: String [required],
        lang: String [required],
        title: String [required],
        theme: String [required],
        difficulte: String [required],
        ordre: i32 [required],
        sort_order: i32 [required],
    }
}
