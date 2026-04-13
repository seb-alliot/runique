//! SeaORM entity `eihwaz_droits` — CRUD access rights in matrix form.
//!
//! A right is linked to several groups via the pivot table `eihwaz_groupes_droits`.
//! - `resource_key = "articles"` + `can_create = true` + `can_read = true`
use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, serde::Serialize, serde::Deserialize)]
#[sea_orm(table_name = "eihwaz_droits")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: crate::utils::pk::Pk,
    /// Unique key of the targeted admin resource (e.g., "articles")
    #[sea_orm(unique)]
    pub resource_key: String,

    // CRUD matrix
    pub can_create: bool,
    pub can_read: bool,
    pub can_update: bool,
    pub can_delete: bool,

    // Row Level Security (Owner only)
    pub can_update_own: bool,
    pub can_delete_own: bool,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

#[async_trait::async_trait]
impl ActiveModelBehavior for ActiveModel {
    /// When a permission is deleted, clear the cache for all users.
    async fn after_delete<C>(self, _db: &C) -> Result<Self, DbErr>
    where
        C: ConnectionTrait,
    {
        crate::auth::permissions_cache::clear_cache();
        Ok(self)
    }
}
