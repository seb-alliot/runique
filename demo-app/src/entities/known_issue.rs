use runique::prelude::*;

model! {
    KnownIssue,
    table: "known_issue",
    pk: id => Pk,
    enums: {
        IssueType: [Manquant, Ajoute, Fix],
    },
    {
        version:     text [required],
        title:       text [required],
        description: richtext [required],
        issue_type:  choice [enum(IssueType), required],
        sort_order:  int [required],
    }
}
