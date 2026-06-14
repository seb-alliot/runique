use runique::prelude::*;

model! {
    Blog,
    table: "blog",
    pk: id => Pk,
    enums: {
        BlogStatus: [Draft="Draft", Published="Published", Archived="Archived"],
    },
    {
        title:   text [required],
        email:   email [required],
        website: url,
        summary: textarea [rows: 3, required],
        content: richtext [rows: 15, required],
        status:     choice [enum(BlogStatus), default: "Draft", required],
        view_count: int [default: 0],
    }
}
