use crate::entities::{blog, users};
use crate::formulaire::{BlogForm, RegisterForm};

admin! {
    users: users::Model => RegisterForm {
        title: "Utilisateurs",
        permissions: ["admin"]
    }
    blog: blog::Model => BlogForm {
        title: "Articles",
        permissions: ["admin"]
    }
}
