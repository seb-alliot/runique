use crate::formulaire::RegisterForm;
use runique::prelude::user;

admin! {
    users: user::Model => RegisterForm {
        title: "Utilisateurs",
        permissions: ["admin"]
    }
}
