use crate::entities::users::schema as users;
use runique::prelude::*;

// Formulaire basé sur le schéma de l'entité Users.
// Seuls les champs listés dans `fields` sont exposés.
#[form(schema = users, fields = [username, email, password])]
pub struct RegisterForm;

impl RegisterForm {
    pub async fn save(
        &self,
        db: &DatabaseConnection,
    ) -> Result<runique::prelude::user::Model, DbErr> {
        use runique::prelude::user::ActiveModel;
        let new_user = ActiveModel {
            username: Set(self.form.get_string("username")),
            email: Set(self.form.get_string("email")),
            password: Set(hash(self.form.get_string("password").as_str()).unwrap_or_default()),
            ..Default::default()
        };
        new_user.insert(db).await
    }
}
