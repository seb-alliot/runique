use crate::entities::users::schema as eihwaz_users_schema;
use runique::prelude::*;

#[form(schema = eihwaz_users_schema, fields = [username, email, password])]
pub struct RegisterForm;

#[form(schema = eihwaz_users_schema, fields = [username, email, is_active, is_staff, is_superuser, roles])]
pub struct UserEditForm;

#[async_trait]
impl RuniqueForm for RegisterForm {
    impl_form_access!(model);

    async fn clean(&mut self) -> Result<(), StrMap> {
        let username = self.get_string("username");
        let email = self.get_string("email");
        let password = self.get_string("password");
        let mut errors = StrMap::new();

        if username.len() < 3 {
            errors.insert(
                "username".to_string(),
                "Username must be at least 3 characters long".to_string(),
            );
        }
        if !email.contains('@') {
            errors.insert("email".to_string(), "Invalid email address".to_string());
        }
        if password.len() < 10 {
            errors.insert(
                "password".to_string(),
                "Password must be at least 10 characters long".to_string(),
            );
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

impl RuniqueForm for UserEditForm {
    impl_form_access!(model);
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
            is_active: Set(true),
            is_superuser: Set(false),
            is_staff: Set(false),
            created_at: Set(Some(chrono::Utc::now().naive_utc())),
            updated_at: Set(Some(chrono::Utc::now().naive_utc())),
            ..Default::default()
        };
        user.insert(db).await
    }
}
