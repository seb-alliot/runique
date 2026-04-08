use crate::entities::eihwaz_users::schema as eihwaz_users_schema;
use runique::prelude::*;

// admin
#[form(schema = eihwaz_users_schema, fields = [username, email, is_active, is_staff, is_superuser, roles])]
pub struct UserEditForm;
impl RuniqueForm for UserEditForm {
    impl_form_access!(model);
}

// inscription
#[form(schema = eihwaz_users_schema, fields = [username, email, password])]
pub struct RegisterForm;
#[async_trait]
impl RuniqueForm for RegisterForm {
    impl_form_access!(model);

    async fn clean(&mut self) -> Result<(), StrMap> {
        let username = self.get_string("username");
        let password = self.get_string("password");
        let mut errors = StrMap::new();

        // Username
        if username.len() < 3 {
            errors.insert(
                "username".to_string(),
                "Le nom d'utilisateur doit faire au moins 3 caractères.".to_string(),
            );
        }
        if username.contains('#') || username.contains('\u{2014}') {
            errors.insert(
                "username".to_string(),
                "Le nom d'utilisateur ne peut pas contenir '#' ou '—'.".to_string(),
            );
        }

        // Mot de passe
        if password.len() < 8 {
            errors.insert(
                "password".to_string(),
                "Le mot de passe doit faire au moins 8 caractères.".to_string(),
            );
        } else if !password.chars().any(|c| c.is_uppercase())
            || !password.chars().any(|c| c.is_ascii_digit())
        {
            errors.insert(
                "password".to_string(),
                "Le mot de passe doit contenir au moins une majuscule et un chiffre.".to_string(),
            );
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

impl RegisterForm {
    pub async fn save(
        &self,
        db: &DatabaseConnection,
    ) -> Result<runique::prelude::user::Model, DbErr> {
        use runique::prelude::user::ActiveModel;
        let user = ActiveModel {
            username: Set(self.form.get_string("username")),
            email: Set(self.form.get_string("email")),
            password: Set(hash(self.form.get_string("password").as_str()).unwrap_or_default()),
            is_active: Set(false),
            is_superuser: Set(false),
            is_staff: Set(false),
            created_at: Set(Some(chrono::Utc::now().naive_utc())),
            updated_at: Set(Some(chrono::Utc::now().naive_utc())),
            ..Default::default()
        };
        user.insert(db).await
    }
}
