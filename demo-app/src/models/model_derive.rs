use runique::prelude::*;
use sea_orm::entity::prelude::*;

// Modèle SeaORM
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, DeriveModelForm)]
#[sea_orm(table_name = "users")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,

    pub username: String,
    pub email: String,
    pub password: String,

    pub bio: Option<String>,
    pub website: Option<String>,

    pub is_active: bool,

    #[sea_orm(column_type = "Timestamp")]
    pub created_at: DateTime,

    #[sea_orm(column_type = "Timestamp")]
    pub updated_at: DateTime,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
