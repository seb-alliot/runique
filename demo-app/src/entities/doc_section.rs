use runique::prelude::*;

model! {
    DocSection,
    table: "doc_section",
    pk: id => Pk,
    enums: {
        SectionTheme: [Demarrage, Web, Database, Security, Admin, Autres],
    },
    {
        slug:       text [required],
        lang:       text [required],
        title:      text [required],
        sort_order: int [required],
        theme:      choice [enum(SectionTheme)],
    }
}
