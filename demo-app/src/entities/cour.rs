use runique::prelude::*;

model! {
    Cour,
    table: "cour",
    pk: id => Pk,
    {
        slug:       text [required],
        lang:       text [required],
        title:      text [required],
        theme:      text [required],
        difficulte: text [required],
        ordre:      int [required],
        sort_order: int [required],
    }
}
