use runique::prelude::*;

model! {
    Blog,
    table: "blog",
    pk: id => Pk,
    fields: {
        title: String [required],
        email: String [required],
        website: String [nullable],
        summary: String [required],
        content: String [required],
    }
}
