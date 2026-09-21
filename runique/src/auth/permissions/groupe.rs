//! SeaORM entity `eihwaz_groupes` — admin permission groups.
use sea_orm::entity::prelude::*;

/// SeaORM model for `eihwaz_groupes`: a named permission group, linked to
/// users via `users_groupes` and to resource permissions via `groupes_droits`.
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, serde::Serialize, serde::Deserialize)]
#[sea_orm(table_name = "eihwaz_groupes")]
pub struct Model {
    // `eihwaz_groupes.id` est toujours INTEGER, indépendant de `Pk` (big-pk/pk-uuid) —
    // cf. migrations_table.rs::create_eihwaz_groupes_table, aucun #[cfg] dessus.
    #[sea_orm(primary_key)]
    pub id: i32,
    pub nom: String,
}

/// SeaORM relations for `eihwaz_groupes`.
#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    /// Memberships of this group in the `eihwaz_users_groupes` junction table.
    #[sea_orm(has_many = "super::users_groupes::Entity")]
    UsersGroupes,
}

impl Related<super::users_groupes::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::UsersGroupes.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
