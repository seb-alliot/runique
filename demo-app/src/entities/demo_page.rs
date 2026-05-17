use runique::prelude::*;

model! {
    DemoPage,
    table: "demo_page",
    pk: id => Pk,
    enums: {
        PageType: [
            Code = "code",
            Form = "form",
            Custom = "custom",
            DocEn = "doc_en",
            DocFr = "doc_fr"
        ],
    },
    {
        category_id: int [required],
        slug:        text [required],
        title:       text [required],
        lead:        text,
        page_type:   choice [enum(PageType), required],
        sort_order:  int [required],
    },
    relations: {
        belongs_to: demo_category via category_id [cascade],
    }
}
