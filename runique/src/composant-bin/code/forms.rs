use runique::prelude::*;


// registration
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
                "Username must be at least 5 characters long.".to_string(),
            );
        }

        if username.contains('#') || username.contains('\u{2014}') {
            errors.insert(
                "username".to_string(),
                "Username cannot contain '#' or '—'.".to_string(),
            );
        }
        // Password
        const SPECIAL: &str = "!@#$%^&*()_+-=[]{}|;':\",./<>?";
        if password.len() < 10 {
            errors.insert(
                "password".to_string(),
                "Password must be at least 10 characters long.".to_string(),
            );
        } else if !password.chars().any(|c| c.is_uppercase())
            || !password.chars().any(|c| c.is_ascii_digit())
            || !password.chars().any(|c| SPECIAL.contains(c))
        {
            errors.insert(
                "password".to_string(),
                "Password must contain at least one uppercase letter, one digit, and one special character (!@#$%...).".to_string(),
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
    ) -> Result<runique::prelude::runique_users::Model, DbErr> {
        use runique::prelude::runique_users::ActiveModel;
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