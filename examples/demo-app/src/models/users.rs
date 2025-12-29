// examples/demo-app/src/models/users.rs
use rusti::sea_orm;
use rusti::sea_orm::entity::prelude::*;
use rusti::DeriveModelForm;
use rusti::impl_objects;
use rusti::Serialize;


#[derive(Clone, Debug, PartialEq, DeriveEntityModel, DeriveModelForm, Serialize)]
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
