use crate::formulaire::user;
use crate::formulaire::RegisterForm;

admin! {
    users: user::Model => RegisterForm {
        title: "Utilisateurs",
        permissions: ["admin"]
    }
}
