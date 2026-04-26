use runique::prelude::*;

model! {
    DocSection,
    table: "doc_section",
    pk: id => Pk,
    enums: {
        SectionTheme: [Demarrage="Demarrage", Web="Web", Database="Database", Security="Security", Admin="Admin", Autres="Autres"]},
    {
        slug:       text [required],
        lang:       text [required],
        title:      text [required],
        sort_order: int [required],
        theme:      choice [enum(SectionTheme)],
    }
}
