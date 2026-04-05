use runique::prelude::*;

model! {
    DemoPage,
    table: "demo_page",
    pk: id => Pk,
    fields: {
        category_id: i32 [required],
        slug: String [required],
        title: String [required],
        lead: String [nullable],
        page_type: String [required],
        sort_order: i32 [required],
    },
    relations: {
        belongs_to: demo_category via category_id [cascade],
    }
}
