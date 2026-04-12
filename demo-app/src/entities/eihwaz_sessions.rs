use runique::prelude::*;

model! {
    EihwazSession,
    table: "eihwaz_sessions",
    pk: id => Pk,
    {
        cookie_id:    text [max_length: 100, unique, required],
        user_id:      int [required],
        session_id:   text [required],
        session_data: text,
        expires_at:   datetime [required],
    },
    relations: {
        belongs_to: eihwaz_users via user_id [cascade],
    }
}
