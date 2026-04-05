use runique::prelude::*;

model! {
    KnownIssue,
    table: "known_issue",
    pk: id => Pk,
    enums: {
        IssueType: [Manquant, Ajoute, Fix],
    },
    fields: {
        version: String [required],
        title: String [required],
        description: String [required],
        issue_type: enum(IssueType) [required],
        sort_order: i32 [required],
    }
}
