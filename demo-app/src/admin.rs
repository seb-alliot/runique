use crate::forms::{Blog, RegisterForm};
use crate::models::users::{self, Entity as UserEntity};
use runique::admin::{build_admin_router, AdminConfig, AdminRegistry};
use runique::prelude::*;

admin! {
    users: UserEntity => RegisterForm {
        title: "Utilisateurs",
        permissions: ["admin"]
    }

    blog: blog::Model => Blog {
        title: "Articles",
        permissions: ["admin", "editor"]
    }
}

// Point d’entrée du router admin pour l’application
pub fn admin_router(registry: &AdminRegistry, config: &AdminConfig) -> Router {
    build_admin_router(registry, config)
}
