use runique::prelude::*;

model! {
    DemoSection,
    table: "demo_section",
    pk: id => Pk,
    fields: {
        page_id: i32 [required],
        title: String [required],
        content: String [nullable],
        sort_order: i32 [required],
    },
    relations: {
        belongs_to: demo_page via page_id [cascade],
    }
}
