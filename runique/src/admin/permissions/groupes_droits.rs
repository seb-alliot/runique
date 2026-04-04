//! Entité SeaORM `eihwaz_groupes_droits` — table de liaison groupe ↔ droit.
use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, serde::Serialize, serde::Deserialize)]
#[sea_orm(table_name = "eihwaz_groupes_droits")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub groupe_id: i32,
    #[sea_orm(primary_key)]
    pub droit_id: i32,
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
    #[sea_orm(
        belongs_to = "super::droit::Entity",
        from = "Column::DroitId",
        to = "super::droit::Column::Id",
        on_delete = "Cascade"
    )]
    Droit,
}

impl Related<super::groupe::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Groupe.def()
    }
}

impl Related<super::droit::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Droit.def()
    }
}

#[async_trait::async_trait]
impl ActiveModelBehavior for ActiveModel {
    async fn after_save<C>(model: Model, db: &C, _insert: bool) -> Result<Model, DbErr>
    where
        C: ConnectionTrait,
    {
        refresh_users_of_groupe(db, model.groupe_id).await;
        Ok(model)
    }

    async fn after_delete<C>(self, db: &C) -> Result<Self, DbErr>
    where
        C: ConnectionTrait,
    {
        if let sea_orm::ActiveValue::Set(gid) | sea_orm::ActiveValue::Unchanged(gid) =
            self.groupe_id.clone()
        {
            refresh_users_of_groupe(db, gid).await;
        }
        Ok(self)
    }
}

/// Rafraîchit le cache de tous les users appartenant à un groupe donné.
async fn refresh_users_of_groupe<C: ConnectionTrait>(db: &C, groupe_id: i32) {
    use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
    let rows = super::users_groupes::Entity::find()
        .filter(super::users_groupes::Column::GroupeId.eq(groupe_id))
        .all(db)
        .await
        .unwrap_or_default();

    for row in rows {
        super::refresh_cache_for_user(db, row.user_id).await;
    }
}
