use crate::entities::users::schema as eihwaz_users_schema;
use runique::prelude::*;

#[form(schema = eihwaz_users_schema, fields = [username, email, password])]
pub struct RegisterForm;

impl RegisterForm {
    async fn clean_fields(&self) -> Result<(), String> {
        let username = self.form.get_string("username");
        let email = self.form.get_string("email");
        let password = self.form.get_string("password");

        if username.len() < 3 {
            return Err("Username must be at least 3 characters long".to_string());
        }

        if !email.contains('@') {
            return Err("Invalid email address".to_string());
        }
        if password.len() < 10 {
            return Err("Password must be at least 10 characters long".to_string());
        }
        Ok(())
    }
    async fn clean(&self) -> Result<(), String> {
        self.clean_fields().await
    }
    pub async fn save(
        &self,
        db: &DatabaseConnection,
    ) -> Result<runique::prelude::user::Model, DbErr> {
        if let Err(e) = self.clean().await {
            return Err(DbErr::Custom(e));
        }
        use runique::prelude::user::ActiveModel;
        let user = ActiveModel {
            username: Set(self.form.get_string("username")),
            email: Set(self.form.get_string("email")),
            password: Set(self.form.get_string("password")),
            is_active: Set(true),
            is_superuser: Set(false),
            is_staff: Set(false),
            ..Default::default()
        };
        user.insert(db).await
    }
}
