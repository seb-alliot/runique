// examples/demo-app/src/models/users.rs
use runique::impl_objects;
use runique::sea_orm;
use runique::sea_orm::entity::prelude::*;
use runique::serde::{Deserialize, Serialize};
use runique::DeriveModelForm;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, DeriveModelForm, Serialize, Deserialize)]
#[sea_orm(table_name = "users")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub username: String,
    pub email: String,
    pub password: String,
    pub age: i32,
    pub created_at: DateTime,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

impl_objects!(Entity);
