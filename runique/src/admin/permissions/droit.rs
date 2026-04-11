//! Entité SeaORM `eihwaz_droits` — droits d'accès CRUD sous forme de matrice.
//!
//! Un droit est lié à plusieurs groupes via la table pivot `eihwaz_groupes_droits`.
//! - `resource_key = "articles"` + `can_create = true` + `can_read = true`
use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, serde::Serialize, serde::Deserialize)]
#[sea_orm(table_name = "eihwaz_droits")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: crate::utils::pk::Pk,
    /// Clé unique de la ressource admin ciblée (ex: "articles")
    #[sea_orm(unique)]
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
pub enum Relation {}

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
