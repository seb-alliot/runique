use runique::prelude::*;

model! {
    Contributions,
    table: "contributions",
    pk: id => Pk,
    enums: {
        ContributionType: [Runique="Runique", Cours="Cours"],
    },
    {
        user_id:           int [required],
        contribution_type: choice [enum(ContributionType), required],
        title:             text [max_length: 200, required],
        content:           richtext [required],
        created_at:        datetime [auto_now],
    },
    relations: {
        belongs_to: eihwaz_users via user_id [cascade],
    }
}
