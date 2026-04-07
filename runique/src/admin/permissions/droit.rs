//! Entité SeaORM `eihwaz_droits` — droits d'accès CRUD sous forme de matrice.
//!
//! Un droit est désormais attaché de manière exclusive à un Groupe.
//! - `resource_key = "articles"` + `can_create = true` + `can_read = true`
use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, serde::Serialize, serde::Deserialize)]
#[sea_orm(table_name = "eihwaz_droits")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: crate::utils::pk::Pk,
    pub groupe_id: crate::utils::pk::Pk,
    /// Clé de la ressource admin ciblée (ex: "articles")
    pub resource_key: String,

    // Matrice CRUD
    pub can_create: bool,
    pub can_read: bool,
    pub can_update: bool,
    pub can_delete: bool,

    // Row Level Security (Propriétaire uniquement)
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

#[async_trait::async_trait]
impl ActiveModelBehavior for ActiveModel {
    /// Quand une permission est supprimée, vide le cache de tous les utilisateurs.
    async fn after_delete<C>(self, _db: &C) -> Result<Self, DbErr>
    where
        C: ConnectionTrait,
    {
        crate::middleware::auth::permissions_cache::clear_cache();
        Ok(self)
    }
}
