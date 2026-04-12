use runique::prelude::*;

model! {
    Blog,
    table: "blog",
    pk: id => Pk,
    {
        title:   text [required],
        email:   email [required],
        website: url,
        summary: textarea [rows: 3, required],
        content: richtext [rows: 15, required],
    }
}
