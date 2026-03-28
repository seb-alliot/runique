use runique::prelude::*;

model! {
    Contributions,
    table: "contributions",
    pk: id => i32,
    fields: {
        user_id: i32 [required],
        contribution_type: String [required],
        title: String [required, max_len(200)],
        content: String [required],
        created_at: datetime [auto_now],
    },
    relations: {
        belongs_to: EihwazUsers via user_id [cascade],
    }
}
