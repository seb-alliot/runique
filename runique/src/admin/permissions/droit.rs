use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, serde::Serialize, serde::Deserialize)]
#[sea_orm(table_name = "eihwaz_droits")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub nom: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::users_droits::Entity")]
    UsersDroits,
    #[sea_orm(has_many = "super::groupes_droits::Entity")]
    GroupesDroits,
}

impl Related<super::users_droits::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::UsersDroits.def()
    }
}

impl Related<super::groupes_droits::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::GroupesDroits.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
