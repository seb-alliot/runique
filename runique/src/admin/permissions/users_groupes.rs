//! Entité SeaORM `eihwaz_users_groupes` — table de liaison utilisateur ↔ groupe.
use crate::utils::pk::Pk;
use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, serde::Serialize, serde::Deserialize)]
#[sea_orm(table_name = "eihwaz_users_groupes")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub user_id: Pk,
    #[sea_orm(primary_key)]
    pub groupe_id: crate::utils::pk::Pk,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "crate::auth::user::Entity",
        from = "Column::UserId",
        to = "crate::auth::user::Column::Id",
        on_delete = "Cascade"
    )]
    User,
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
