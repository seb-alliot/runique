use runique::prelude::*;

model! {
    ChangelogEntry,
    table: "changelog_entry",
    pk: id => i32,
    enums: {
        ChangelogCategory: pg [Fix="Fix", Feature="Feature", Ajoute="Ajouté"],
    },
    fields: {
        version: String [required],
        release_date: String [required],
        category: enum(ChangelogCategory) [required],
        title: String [required],
        description: String [required],
        sort_order: i32 [required],
    }
}
