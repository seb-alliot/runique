use runique::prelude::*;

model! {
    Contributions,
    table: "contributions",
    pk: id => i32,
    enums: {
        ContributionType: pg [Runique, Cours],
    },
    fields: {
        user_id: i32 [required],
        contribution_type: enum(ContributionType) [required],
        title: String [required, max_len(200)],
        content: String [required],
        created_at: datetime [auto_now],
    },
    relations: {
        belongs_to: eihwaz_users via user_id [cascade],
    }
}
