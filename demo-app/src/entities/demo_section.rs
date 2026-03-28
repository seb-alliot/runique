use runique::prelude::*;

model! {
    DemoSection,
    table: "demo_section",
    pk: id => i32,
    fields: {
        page_id: i32 [required],
        title: String [required],
        content: String [nullable],
        sort_order: i32 [required],
    },
    relations: {
        belongs_to: DemoPage via page_id [cascade],
    }
}
