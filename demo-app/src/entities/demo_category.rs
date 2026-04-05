use runique::prelude::*;

model! {
    DemoCategory,
    table: "demo_category",
    pk: id => Pk,
    fields: {
        title: String [required],
        back_link_url: String [nullable],
        back_link_label: String [nullable],
        sort_order: i32 [required],
    }
}
