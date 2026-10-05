use runique::prelude::*;

model! {
    UsersBooster,
    table: "users_booster",
    pk: id => Pk,
    {
        username:  text [required],
        email:     email [required],
        password:  password [required],
        bio:       textarea [nullable],
        website:   url [nullable],
        is_active: bool [required],
    }
}
