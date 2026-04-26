use runique::prelude::*;

model! {
    PageDocLink,
    table: "page_doc_link",
    pk: id => Pk,
    {
        page_id:    int [required],
        label:      text [required],
        url:        url [required],
        link_type:  text [required],
        sort_order: int [required],
    },
    relations: {
        belongs_to: demo_page via page_id [cascade],
    }
}
