use runique::prelude::*;

model! {
    DocPage,
    table: "doc_page",
    pk: id => Pk,
    {
        section_id: int [required],
        slug:       text [required],
        lang:       text [required],
        title:      text [required],
        lead:       text,
        sort_order: int [required],
    },
    relations: {
        belongs_to: doc_section via section_id [cascade],
    }
}
