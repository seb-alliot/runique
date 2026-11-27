use crate::backend::lang::visitor_lang;
use runique::axum::http::HeaderMap;
use runique::prelude::*;

// inscription
#[form(schema = runique_users, fields = [username, email, password])]
pub struct RegisterForm;
#[async_trait]
impl RuniqueForm for RegisterForm {
    async fn clean(&mut self) -> Result<(), StrMap> {
        let username = self.cleaned::<String>("username").unwrap_or_default();
        let password = self.cleaned::<String>("password").unwrap_or_default();
        let mut errors = StrMap::new();

        if username.len() < 5 {
            errors.insert(
                "username".to_string(),
                "The username must be at least 5 characters long.".to_string(),
            );
        }

        // The username is written unescaped in the activation email: `clean()` runs
        // after entities are decoded, so `<` is seen here even if sent as `&lt;`.
        if username
            .chars()
            .any(|c| matches!(c, '#' | '\u{2014}' | '<' | '>' | '&' | '"' | '\''))
        {
            errors.insert(
                "username".to_string(),
                "The username cannot contain #, —, <, >, &, \" or '.".to_string(),
            );
        }
        // Mot de passe
        const SPECIAL: &str = "!@#$%^&*()_+-=[]{}|;':\",./<>?";
        if password.len() < 12 {
            errors.insert(
                "password".to_string(),
                "The password must be at least 12 characters long.".to_string(),
            );
        } else if !password.chars().any(|c| c.is_uppercase())
            || !password.chars().any(|c| c.is_ascii_digit())
            || !password.chars().any(|c| SPECIAL.contains(c))
        {
            errors.insert(
                "password".to_string(),
                "The password must contain at least one uppercase letter, one digit and one special character.".to_string(),
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
    pub async fn save(&self, db: &ADb) -> Result<runique::prelude::user::Model, DbErr> {
        use runique::prelude::user::ActiveModel;
        #[allow(unused_mut)]
        let mut user = ActiveModel {
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
        // Uuid PKs are never DB auto-increment — must be generated
        // application-side, unlike i32/i64 which SeaORM fills in itself.
        #[cfg(feature = "pk-uuid")]
        {
            user.id = Set(::sea_orm::prelude::Uuid::now_v7());
        }
        user.insert(db).await
    }
}

impl RegisterForm {
    pub fn in_language(mut self, headers: &HeaderMap) -> Self {
        if visitor_lang(headers) == "fr" {
            self.forms_mut()
                .field_label("username", "Nom d'utilisateur")
                .field_label("password", "Mot de passe");
        }
        self
    }
}
