use runique::prelude::*;

model! {
    DemoCategory,
    table: "demo_category",
    pk: id => Pk,
    {
        title:           text [required],
        back_link_url:   url [nullable],
        back_link_label: text [nullable],
        sort_order:      int [required],
    }
}
