//! SeaORM entity `eihwaz_users_groupes` — user ↔ group junction table.
use crate::utils::pk::Pk;
use sea_orm::entity::prelude::*;

/// SeaORM model for `eihwaz_users_groupes`, the user-to-group junction table
/// (composite primary key `(user_id, groupe_id)`). Saving or deleting a row
/// refreshes the cached permissions for the affected user — see
/// `ActiveModelBehavior` below.
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, serde::Serialize, serde::Deserialize)]
#[sea_orm(table_name = "eihwaz_users_groupes")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub user_id: Pk,
    // Toujours INTEGER, indépendant de `Pk` — cf. groupe.rs::Model::id.
    #[sea_orm(primary_key)]
    pub groupe_id: i32,
}

/// SeaORM relations for `eihwaz_users_groupes`.
#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    /// The member user; deleting the user cascades to this membership row.
    #[sea_orm(
        belongs_to = "crate::auth::user::Entity",
        from = "Column::UserId",
        to = "crate::auth::user::Column::Id",
        on_delete = "Cascade"
    )]
    User,
    /// The group this membership grants; deleting the group cascades here too.
    #[sea_orm(
        belongs_to = "super::groupe::Entity",
        from = "Column::GroupeId",
        to = "super::groupe::Column::Id",
        on_delete = "Cascade"
    )]
    Groupe,
}

impl Related<crate::auth::user::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::User.def()
    }
}

impl Related<super::groupe::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Groupe.def()
    }
}

#[async_trait::async_trait]
impl ActiveModelBehavior for ActiveModel {
    async fn after_save<C>(model: Model, db: &C, _insert: bool) -> Result<Model, DbErr>
    where
        C: ConnectionTrait,
    {
        super::refresh_cache_for_user(db, model.user_id).await;
        Ok(model)
    }

    async fn after_delete<C>(self, db: &C) -> Result<Self, DbErr>
    where
        C: ConnectionTrait,
    {
        if let sea_orm::ActiveValue::Set(uid) | sea_orm::ActiveValue::Unchanged(uid) =
            self.user_id.clone()
        {
            super::refresh_cache_for_user(db, uid).await;
        }
        Ok(self)
    }
}
