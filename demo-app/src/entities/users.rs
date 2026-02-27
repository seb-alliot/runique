use runique::prelude::*;

model! {
    EihwazUsers,
    table: "eihwaz_users",
    pk: id => i32,
    fields: {
        username: String [required, max_len(150), unique],
        email: String [required, unique],
        password: String [required, max_len(128)],
        is_active: bool [required],
        is_staff: bool [required],
        is_superuser: bool [required],
        roles: String [nullable],
        created_at: datetime [auto_now],
        updated_at: datetime [auto_now_update],
    }
}
