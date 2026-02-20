use crate::entities::users::eihwaz_users_schema;
use runique::prelude::*;

#[form(schema = eihwaz_users_schema, fields = [username, email, password, _password])]
pub struct RegisterForm;

impl RegisterForm {
    pub async fn save(
        &self,
        db: &DatabaseConnection,
    ) -> Result<runique::prelude::user::Model, DbErr> {
        let mut form = RegisterForm::from_form(self.get_form().clone());
        let _ = form.clean().await;
        use runique::prelude::user::ActiveModel;
        let new_user = ActiveModel {
            username: Set(self.form.get_string("username")),
            email: Set(self.form.get_string("email")),
            password: Set(self.form.get_string("password")),
            ..Default::default()
        };
        new_user.insert(db).await
    }
}
