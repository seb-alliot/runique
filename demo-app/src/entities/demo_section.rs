use runique::prelude::*;

model! {
    DemoSection,
    table: "demo_section",
    pk: id => Pk,
    {
        page_id:    int [required],
        title:      text [required],
        content:    text,
        sort_order: int [required],
    },
    relations: {
        belongs_to: demo_page via page_id [cascade],
    }
}
