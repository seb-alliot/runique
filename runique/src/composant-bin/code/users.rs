use runique::prelude::*;

model! {
    Users,
    table: "users",
    pk: id => i32,
    fields: {
        username: String [required, max_len(150), unique],
        email: String [required, unique],
        password: String [required, max_len(128)],
        created_at: datetime [auto_now],
        updated_at: datetime [auto_now_update],
    }
}