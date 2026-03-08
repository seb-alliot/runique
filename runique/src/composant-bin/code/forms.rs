use crate::entities::users::schema as users;
use runique::prelude::*;

// Schema-based form — fields are derived directly from the entity schema.
// Only list the fields you want to expose; the macro handles the rest
// (field registration, Serialize, RuniqueForm implementation).
#[form(schema = users, fields = [username, email, password])]
pub struct RegisterForm;

impl RegisterForm {
    pub async fn save(
        &self,
        db: &DatabaseConnection,
    ) -> Result<crate::entities::users::Model, DbErr> {
        use runique::prelude::user::ActiveModel;
        let new_user = ActiveModel {
            username: Set(self.form.get_string("username")),
            email: Set(self.form.get_string("email")),
            password: Set(self.form.get_string("password")),
            ..Default::default()
        };
        let inserted = new_user.insert(db).await?;
        // Convert runique::middleware::user::Model to users::Model
        let users_model = crate::entities::users::Model {
            id: inserted.id,
            username: inserted.username,
            email: inserted.email,
            password: inserted.password,
            // Add other fields as needed
        };
        Ok(users_model)
    }
}