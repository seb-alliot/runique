//! Table `eihwaz_groupes_droits` — group permissions on a resource.
//! Composite PK: (groupe_id, resource_key).
use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, serde::Serialize)]
#[sea_orm(table_name = "eihwaz_groupes_droits")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub groupe_id: crate::utils::pk::Pk,
    #[sea_orm(primary_key, auto_increment = false)]
    pub resource_key: String,
    pub can_create: bool,
    pub can_read: bool,
    pub can_update: bool,
    pub can_delete: bool,
    pub can_update_own: bool,
    pub can_delete_own: bool,
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
}

impl Related<super::groupe::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Groupe.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
