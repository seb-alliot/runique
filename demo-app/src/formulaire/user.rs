use runique::prelude::*;
use serde::Serialize;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "eihwaz_users")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub username: String,
    pub email: String,
    pub password: String,

    pub is_active: bool,
    pub is_staff: bool,
    pub is_superuser: bool,

    pub roles: Option<String>,

    pub created_at: Option<NaiveDateTime>,
    pub updated_at: Option<NaiveDateTime>,
}

impl Model {
    #[allow(dead_code)]
    pub fn get_roles(&self) -> Vec<String> {
        self.roles
            .as_deref()
            .and_then(|r| serde_json::from_str(r).ok())
            .unwrap_or_default()
    }

    #[allow(dead_code)]
    pub fn can_access_admin(&self) -> bool {
        self.is_active && (self.is_staff || self.is_superuser)
    }
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
impl_objects!(Entity);

// --- FORMULAIRE D'INSCRIPTION ---
#[derive(Serialize, Debug, Clone)]
#[serde(transparent)]
pub struct RegisterForm {
    pub form: Forms,
}

impl RuniqueForm for RegisterForm {
    fn register_fields(form: &mut Forms) {
        form.field(
            &TextField::text("username")
                .label("Entrez votre nom d'utilisateur")
                .required(),
        );

        form.field(
            &TextField::email("email")
                .label("Entrez votre email")
                .required(),
        );

        form.field(
            &TextField::password("password")
                .label("Entrez un mot de passe")
                .required(),
        );
    }

    impl_form_access!();
}

impl RegisterForm {
    pub async fn save(
        &self,
        db: &DatabaseConnection,
    ) -> Result<crate::formulaire::user::Model, DbErr> {
        use crate::formulaire::user::ActiveModel;
        let new_user = ActiveModel {
            username: Set(self.form.get_string("username")),
            email: Set(self.form.get_string("email")),
            password: Set(self.form.get_string("password")),
            ..Default::default()
        };

        new_user.insert(db).await
    }
}
