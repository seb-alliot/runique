use runique::prelude::*;

model! {
    Blog,
    table: "blog",
    pk: id => i32,
    fields: {
        title: String [required],
        email: String [required],
        website: String [nullable],
        summary: String [required],
        content: String [required],
    }
}
