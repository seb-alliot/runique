//! Table `eihwaz_groupes_droits` — group permissions on a resource.
//! Composite PK: (groupe_id, resource_key).
use sea_orm::entity::prelude::*;

/// SeaORM model for `eihwaz_groupes_droits`: the CRUD permission flags a
/// group holds on one resource, keyed by the composite primary key
/// `(groupe_id, resource_key)`. `can_update_own`/`can_delete_own` scope the
/// update/delete grant to rows owned by the acting user. Read on every admin
/// request (`pull_groupes_db`), so a change applies to the next one.
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, serde::Serialize)]
#[sea_orm(table_name = "eihwaz_groupes_droits")]
pub struct Model {
    // Toujours INTEGER, indépendant de `Pk` — cf. groupe.rs::Model::id.
    #[sea_orm(primary_key, auto_increment = false)]
    pub groupe_id: i32,
    #[sea_orm(primary_key, auto_increment = false)]
    pub resource_key: String,
    pub can_create: bool,
    pub can_read: bool,
    pub can_update: bool,
    pub can_delete: bool,
    pub can_update_own: bool,
    pub can_delete_own: bool,
}

/// SeaORM relations for `eihwaz_groupes_droits`.
#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    /// The group this permission row belongs to; deleting the group cascades
    /// to its permission rows.
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
