use runique::prelude::*;

model! {
    UsersBooster,
    table: "users_booster",
    pk: id => Pk,
    {
        username:  text [required],
        email:     email [required],
        password:  password [required],
        bio:       textarea,
        website:   url,
        is_active: bool [required],
    }
}
