use crate::entities::users;
use crate::formulaire::RegisterForm;

admin! {
    users: users::Model => RegisterForm {
        title: "Utilisateurs",
        permissions: ["admin"]
    }
}
