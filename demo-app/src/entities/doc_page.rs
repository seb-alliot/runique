use runique::prelude::*;

model! {
    DocPage,
    table: "doc_page",
    pk: id => Pk,
    fields: {
        section_id: i32 [required],
        slug: String [required],
        lang: String [required],
        title: String [required],
        lead: String [nullable],
        sort_order: i32 [required],
    },
    relations: {
        belongs_to: doc_section via section_id [cascade],
    }
}
