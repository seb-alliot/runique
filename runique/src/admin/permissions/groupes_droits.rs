use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, serde::Serialize, serde::Deserialize)]
#[sea_orm(table_name = "eihwaz_groupes_droits")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub groupe_id: i32,
    #[sea_orm(primary_key)]
    pub droit_id: i32,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::groupe::Entity",
        from = "Column::GroupeId",
        to = "super::groupe::Column::Id",
        on_delete = "Cascade"
    )]
    Groupe,
    #[sea_orm(
        belongs_to = "super::droit::Entity",
        from = "Column::DroitId",
        to = "super::droit::Column::Id",
        on_delete = "Cascade"
    )]
    Droit,
}

impl Related<super::groupe::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Groupe.def()
    }
}

impl Related<super::droit::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Droit.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
