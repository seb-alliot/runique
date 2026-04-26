//! SeaORM entity `eihwaz_groupes` — admin permission groups.
use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, serde::Serialize, serde::Deserialize)]
#[sea_orm(table_name = "eihwaz_groupes")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: crate::utils::pk::Pk,
    pub nom: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::users_groupes::Entity")]
    UsersGroupes,
}

impl Related<super::users_groupes::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::UsersGroupes.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
