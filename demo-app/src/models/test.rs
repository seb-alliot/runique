use runique::impl_objects;
use runique::prelude::*;
use runique::sea_orm;
use runique::sea_orm::entity::prelude::*;
use runique::serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "test_fields")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,

    pub username: String,    // Match TextField
    pub email: String,       // Match EmailField
    pub website: String,     // Match URLField
    pub description: String, // Match TextAreaField / RichTextField
    pub password: String,    // Match PasswordField (stockage du hash)
    pub created_at: DateTime,

}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {
    fn new() -> Self {
        Self {
            created_at: Set(chrono::Utc::now().naive_utc()),
            ..Default::default()
        }
    }
}

impl_objects!(Entity);
