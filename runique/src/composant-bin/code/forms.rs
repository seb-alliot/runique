use crate::entities::users::schema as users;
use runique::prelude::*;

// Form based on the Users entity schema.
// Only the fields listed in `fields` are exposed.
#[form(schema = users, fields = [username, email, password])]
pub struct RegisterForm;

impl RegisterForm {
    pub async fn save(
        &self,
        db: &DatabaseConnection,
    ) -> Result<runique::prelude::user::Model, DbErr> {
        use runique::prelude::user::ActiveModel;
        let new_user = ActiveModel {
            username: Set(self.cleaned_string("username").unwrap_or_default()),
            email: Set(self.cleaned_string("email").unwrap_or_default()),
            password: Set(self.cleaned_string("password").unwrap_or_default()),
            ..Default::default()
        };
        new_user.insert(db).await
    }
}
