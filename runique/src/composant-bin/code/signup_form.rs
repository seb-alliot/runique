use runique::prelude::*;
use runique::sea_orm::TransactionTrait;

/// The sign-up form, built from the account model: checks the username and
/// the password strength, then saves a new account, activated at once.
#[form(schema = runique_users, fields = [username, email, password])]
pub struct SignupForm;

#[async_trait]
impl RuniqueForm for SignupForm {
    async fn clean(&mut self) -> Result<(), StrMap> {
        let username = self.cleaned::<String>("username").unwrap_or_default();
        let password = self.cleaned::<String>("password").unwrap_or_default();
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
        const SPECIAL: &str = "!@#$%^&*()_+-=[]{}|;':\",./<>?";
        if password.len() < 10 {
            errors.insert(
                "password".to_string(),
                "Password must be at least 10 characters long.".to_string(),
            );
        } else if !password.chars().any(|c| c.is_uppercase())
            || !password.chars().any(|c| c.is_lowercase())
            || !password.chars().any(|c| c.is_ascii_digit())
            || !password.chars().any(|c| SPECIAL.contains(c))
        {
            errors.insert(
                "password".to_string(),
                "Password must contain at least one uppercase letter, one lowercase letter, one digit, and one special character (!@#$%...).".to_string(),
            );
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

impl SignupForm {
    pub async fn save(
        &self,
        db: &ADb,
    ) -> Result<runique::prelude::runique_users::Model, DbErr> {
        use runique::prelude::runique_users::ActiveModel;
        let user = ActiveModel {
            username: Set(self.cleaned::<String>("username").unwrap_or_default()),
            email: Set(self.cleaned::<String>("email").unwrap_or_default()),
            password: Set(self.cleaned::<String>("password").unwrap_or_default()),
            is_active: Set(false),
            is_superuser: Set(false),
            is_dirigeant: Set(false),
            is_admin: Set(false),
            created_at: Set(Some(chrono::Utc::now().naive_utc())),
            updated_at: Set(Some(chrono::Utc::now().naive_utc())),
            ..Default::default()
        };
        // Created and activated in one transaction: an activation that fails
        // takes the new row back with it, so no inactive account is left
        // holding the username and the email.
        let txn = db.begin().await?;
        // A UUID key, if the project uses them, is set by the account model itself.
        let user = user.insert(&txn).await?;

        // An account is created inactive and can't sign in until it's
        // activated: deciding when is business logic. Here, at once. To
        // confirm the email first, send a link instead and call
        // `activate_account` when it's opened.
        let user = BuiltinUserEntity::activate_account(&txn, user.id)
            .await?
            .ok_or_else(|| DbErr::Custom("the new account was already activated".into()))?;
        txn.commit().await?;
        Ok(user)
    }
}