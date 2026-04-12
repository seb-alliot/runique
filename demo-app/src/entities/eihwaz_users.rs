use runique::prelude::*;

model! {
    EihwazUsers,
    table: "eihwaz_users",
    pk: id => Pk,
    {
        username:     text [max_length: 150, unique, required],
        email:        email [unique, required],
        password:     password [max_length: 128, required],
        is_active:    bool [required],
        is_staff:     bool [required],
        is_superuser: bool [required],
        roles:        text,
        created_at:   datetime [auto_now],
        updated_at:   datetime [auto_now_update],
    }
}
