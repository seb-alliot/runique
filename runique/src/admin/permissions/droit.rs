//! Entité SeaORM `eihwaz_droits` — droits d'accès individuels pour les utilisateurs admin.
//!
//! Un droit peut être global (`resource_key = NULL`) ou scopé à une ressource admin.
//! - `resource_key = "blog"` + `access_type = "view"` → voir la ressource blog dans la nav
//! - `resource_key = "blog"` + `access_type = "write"` → create/edit/delete sur blog
use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, serde::Serialize, serde::Deserialize)]
#[sea_orm(table_name = "eihwaz_droits")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub nom: String,
    /// Clé de la ressource admin ciblée — `None` = droit global
    pub resource_key: Option<String>,
    /// Type d'accès sur la ressource — `None` pour les droits globaux
    pub access_type: Option<String>,
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

#[async_trait::async_trait]
impl ActiveModelBehavior for ActiveModel {
    /// Quand un droit est supprimé, vide le cache de tous les utilisateurs.
    /// Chaque user rechargera ses permissions depuis la DB à la prochaine requête.
    async fn after_delete<C>(self, _db: &C) -> Result<Self, DbErr>
    where
        C: ConnectionTrait,
    {
        crate::middleware::auth::permissions_cache::clear_cache();
        Ok(self)
    }
}
