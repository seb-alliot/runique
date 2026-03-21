use runique::prelude::*;

model! {
    DocSection,
    table: "doc_section",
    pk: id => i32,
    fields: {
        slug: String [required],
        lang: String [required],
        title: String [required],
        sort_order: i32 [required],
    }
}
