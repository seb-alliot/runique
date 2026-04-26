use runique::prelude::*;

model! {
    DemoCategory,
    table: "demo_category",
    pk: id => Pk,
    {
        title:           text [required],
        back_link_url:   url,
        back_link_label: text,
        sort_order:      int [required],
    }
}
