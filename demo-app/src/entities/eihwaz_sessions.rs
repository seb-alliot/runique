use runique::prelude::*;

model! {
    EihwazSession,
    table: "eihwaz_sessions",
    pk: id => i32,
    fields: {
        cookie_id: String [required, max_len(100), unique],
        user_id: i32 [required],
        session_id: String [required],
        session_data: String [nullable],
        expires_at: datetime [required],
    },
    relations: {
        belongs_to: eihwaz_users via user_id [cascade],
    }
}
