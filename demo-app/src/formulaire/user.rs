use runique::prelude::*;

// admin
#[form(schema = runique_users, fields = [username, email, is_active, is_staff, is_superuser, roles])]
pub struct UserEditForm;
impl RuniqueForm for UserEditForm {
    impl_form_access!(model);
}

// inscription
#[form(schema = runique_users, fields = [username, email, password])]
pub struct RegisterForm;
#[async_trait]
impl RuniqueForm for RegisterForm {
    impl_form_access!(model);

    async fn clean(&mut self) -> Result<(), StrMap> {
        let username = self.cleaned_string("username").unwrap_or_default();
        let password = self.cleaned_string("password").unwrap_or_default();
        let mut errors = StrMap::new();

        if username.len() < 5 {
            errors.insert(
                "username".to_string(),
                "Le nom d'utilisateur doit faire au moins 5 caractères.".to_string(),
            );
        }

        if username.contains('#') || username.contains('\u{2014}') {
            errors.insert(
                "username".to_string(),
                "Le nom d'utilisateur ne peut pas contenir '#' ou '—'.".to_string(),
            );
        }
        // Mot de passe
        const SPECIAL: &str = "!@#$%^&*()_+-=[]{}|;':\",./<>?";
        if password.len() < 10 {
            errors.insert(
                "password".to_string(),
                "Le mot de passe doit faire au moins 10 caractères.".to_string(),
            );
        } else if !password.chars().any(|c| c.is_uppercase())
            || !password.chars().any(|c| c.is_ascii_digit())
            || !password.chars().any(|c| SPECIAL.contains(c))
        {
            errors.insert(
                "password".to_string(),
                "Le mot de passe doit contenir au moins une majuscule, un chiffre et un caractère spécial (!@#$%...).".to_string(),
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
            username: Set(self.cleaned_string("username").unwrap_or_default()),
            email: Set(self.cleaned_string("email").unwrap_or_default()),
            password: Set(self.cleaned_string("password").unwrap_or_default()),
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
