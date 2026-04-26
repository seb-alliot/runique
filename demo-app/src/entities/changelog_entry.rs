use runique::prelude::*;

model! {
    ChangelogEntry,
    table: "changelog_entry",
    pk: id => Pk,
    enums: {
        ChangelogCategory: [Fix="Fix", Feature="Feature", Ajoute="Ajouté"],
    },
    {
        version:      text [required],
        release_date: text [required],
        category:     choice [enum(ChangelogCategory), required],
        title:        text [required],
        description:  richtext [rows: 6, required],
        sort_order:   int [required],
    }
}
