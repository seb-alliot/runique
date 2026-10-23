//! SeaORM entity `eihwaz_groupes` — admin permission groups.
use crate::utils::pk::Pk;
use sea_orm::entity::prelude::*;

/// SeaORM model for `eihwaz_groupes`: a named permission group, linked to
/// users via `users_groupes` and to resource permissions via `groupes_droits`.
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, serde::Serialize, serde::Deserialize)]
#[sea_orm(table_name = "eihwaz_groupes")]
pub struct Model {
    // `eihwaz_groupes.id` est toujours INTEGER, indépendant de `Pk` (big-pk/pk-uuid) —
    // cf. auth/migrations_table.rs::create_eihwaz_groupes_table, aucun #[cfg] dessus.
    #[sea_orm(primary_key)]
    pub id: i32,
    pub nom: String,
    pub is_staff: bool,
    pub responsable_id: Pk,
    pub repris_par: Option<Pk>,
    pub is_active: bool,
}

/// SeaORM relations for `eihwaz_groupes`.
#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    /// Memberships of this group in the `eihwaz_users_groupes` junction table.
    #[sea_orm(has_many = "super::users_groupes::Entity")]
    UsersGroupes,
    /// The account answering for the group; can't be deleted while it does.
    #[sea_orm(
        belongs_to = "crate::auth::user::Entity",
        from = "Column::ResponsableId",
        to = "crate::auth::user::Column::Id",
        on_delete = "Restrict"
    )]
    Responsable,
    #[sea_orm(
        belongs_to = "crate::auth::user::Entity",
        from = "Column::ReprisPar",
        to = "crate::auth::user::Column::Id",
        on_delete = "SetNull"
    )]
    ReprisPar,
}

impl Related<super::users_groupes::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::UsersGroupes.def()
    }
}
impl Related<crate::auth::user::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Responsable.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
