use runique::prelude::*;

model! {
    ChangelogEntry,
    table: "changelog_entry",
    pk: id => i32,
    fields: {
        version: String [required],
        release_date: String [required],
        category: String [required],
        title: String [required],
        description: String [required],
        sort_order: i32 [required],
    }
}
