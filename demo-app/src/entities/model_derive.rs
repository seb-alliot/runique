use runique::prelude::*;

model! {
    UsersBooster,
    table: "users_booster",
    pk: id => i32,
    fields: {
        username: String [required],
        email: String [required],
        password: String [required],
        bio: String [nullable],
        website: String [nullable],
        is_active: bool [required],
    }
}
