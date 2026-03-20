use runique::prelude::*;

model! {
    KnownIssue,
    table: "known_issue",
    pk: id => i32,
    fields: {
        version: String [required],
        title: String [required],
        description: String [required],
        issue_type: String [required],
        sort_order: i32 [required],
    }
}
