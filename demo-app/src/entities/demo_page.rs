use runique::prelude::*;

model! {
    DemoPage,
    table: "demo_page",
    pk: id => Pk,
    {
        category_id: int [required],
        slug:        text [required],
        title:       text [required],
        lead:        text,
        page_type:   text [required],
        sort_order:  int [required],
    },
    relations: {
        belongs_to: demo_category via category_id [cascade],
    }
}
