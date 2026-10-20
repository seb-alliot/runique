//! Table `eihwaz_groupes_droits`: the rights of a group or of an admin
//! account, on a whole table (`colonne = ""`) or on one of its columns.
use crate::utils::pk::Pk;
use sea_orm::entity::prelude::*;

/// SeaORM model for `eihwaz_groupes_droits`. Exactly one of `groupe_id` /
/// `user_id` is set (CHECK). A column row only carries read/create/update
/// (CHECK). Read on every admin request, so a change applies to the next one.
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, serde::Serialize)]
#[sea_orm(table_name = "eihwaz_groupes_droits")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    // Toujours INTEGER, indépendant de `Pk` — cf. groupe.rs::Model::id.
    pub groupe_id: Option<i32>,
    pub user_id: Option<Pk>,
    pub resource_key: String,
    pub colonne: String,
    pub can_create: bool,
    pub can_read: bool,
    pub can_read_own: bool,
    pub can_update: bool,
    pub can_update_own: bool,
    pub can_delete: bool,
    pub can_delete_own: bool,
}

/// SeaORM relations for `eihwaz_groupes_droits`.
#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    /// The group holding the row; deleting the group cascades to its rows.
    #[sea_orm(
        belongs_to = "super::groupe::Entity",
        from = "Column::GroupeId",
        to = "super::groupe::Column::Id",
        on_delete = "Cascade"
    )]
    Groupe,
    /// The admin account holding the row; deleting the account cascades too.
    #[sea_orm(
        belongs_to = "crate::auth::user::Entity",
        from = "Column::UserId",
        to = "crate::auth::user::Column::Id",
        on_delete = "Cascade"
    )]
    User,
}

impl Related<super::groupe::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Groupe.def()
    }
}

impl Related<crate::auth::user::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::User.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
