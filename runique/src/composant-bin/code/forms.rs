use runique::prelude::*;
use serde::Serialize;

// --- FORMULAIRE D'INSCRIPTION ---
#[derive(Serialize, Debug, Clone)]
#[serde(transparent)]
pub struct RegisterForm {
    pub form: Forms,
}

impl RuniqueForm for RegisterForm {
    fn register_fields(form: &mut Forms) {
        form.field(
            &TextField::text("username")
                .label("Entrez votre nom d'utilisateur")
                .required(),
        );

        form.field(
            &TextField::email("email")
                .label("Entrez votre email")
                .required(),
        );

        form.field(
            &TextField::password("password")
                .label("Entrez un mot de passe")
                .required(),
        );
    }

    impl_form_access!();
}

impl RegisterForm {
    pub async fn save(
        &self,
        db: &DatabaseConnection,
    ) -> Result<crate::models::users::Model, DbErr> {
        use crate::models::users as users_mod;
        let new_user = users_mod::ActiveModel {
            username: Set(self.form.get_string("username")),
            email: Set(self.form.get_string("email")),
            password: Set(self.form.get_string("password")),
            ..Default::default()
        };

        new_user.insert(db).await
    }
}