#[path = "../target/runique/admin/generated.rs"]
#[allow(warnings)]
pub mod generated;

use crate::forms::RegisterForm;
use crate::models::users;

admin! {
    users: users::Model => RegisterForm {
        title: "Utilisateurs",
        permissions: ["admin"]
    }
}
