//! Entité utilisateur built-in de Runique (table `eihwaz_users`).
pub use crate::middleware::auth::{default_auth::UserEntity, user_trait::RuniqueUser};
use crate::utils::pk::UserId;
use crate::{impl_objects, search};
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, DatabaseConnection, EntityTrait, entity::prelude::*,
};

// ─── Modèle SeaORM ───────────────────────────────────────────────────────────

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, serde::Serialize, serde::Deserialize)]
#[sea_orm(table_name = "eihwaz_users")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: UserId,
    pub username: String,
    pub email: String,
    pub password: String,
    pub is_active: bool,
    pub is_staff: bool,
    pub is_superuser: bool,
    pub created_at: Option<chrono::NaiveDateTime>,
    pub updated_at: Option<chrono::NaiveDateTime>,
}

impl_objects!(Entity);

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "crate::admin::permissions::users_droits::Entity")]
    UsersDroits,
    #[sea_orm(has_many = "crate::admin::permissions::users_groupes::Entity")]
    UsersGroupes,
    #[sea_orm(has_many = "crate::middleware::session::session_db::Entity")]
    Sessions,
}

impl Related<crate::admin::permissions::users_droits::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::UsersDroits.def()
    }
}

impl Related<crate::admin::permissions::users_groupes::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::UsersGroupes.def()
    }
}

impl Related<crate::middleware::session::session_db::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Sessions.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

// ─── RuniqueUser ─────────────────────────────────────────────────────────────
impl RuniqueUser for Model {
    fn user_id(&self) -> UserId {
        self.id
    }
    fn username(&self) -> &str {
        &self.username
    }
    fn email(&self) -> &str {
        &self.email
    }
    fn password_hash(&self) -> &str {
        &self.password
    }
    fn is_active(&self) -> bool {
        self.is_active
    }
    fn is_staff(&self) -> bool {
        self.is_staff
    }
    fn is_superuser(&self) -> bool {
        self.is_superuser
    }
}

// ─── UserEntity ──────────────────────────────────────────────────────────────
pub struct BuiltinUserEntity;

#[async_trait::async_trait]
impl UserEntity for BuiltinUserEntity {
    type Model = Model;

    async fn find_by_id(db: &DatabaseConnection, id: UserId) -> Option<Self::Model> {
        Entity::find_by_id(id).one(db).await.ok().flatten()
    }

    async fn find_by_username(db: &DatabaseConnection, username: &str) -> Option<Self::Model> {
        search!(Entity => Username eq username)
            .first(db)
            .await
            .ok()
            .flatten()
    }

    async fn find_by_email(db: &DatabaseConnection, email: &str) -> Option<Self::Model> {
        search!(Entity => Email eq email)
            .first(db)
            .await
            .ok()
            .flatten()
    }

    async fn update_password(
        db: &DatabaseConnection,
        email: &str,
        new_hash: &str,
    ) -> Result<(), sea_orm::DbErr> {
        let user = search!(Entity => Email eq email)
            .first(db)
            .await?
            .ok_or(sea_orm::DbErr::RecordNotFound("User not found".into()))?;

        let mut active: ActiveModel = user.into();
        active.password = Set(new_hash.to_string());
        active.is_active = Set(true);
        active.update(db).await?;
        Ok(())
    }
}

// ─── Alias pratique ───────────────────────────────────────────────────────────
pub type RuniqueAdminAuth =
    crate::middleware::auth::default_auth::DefaultAdminAuth<BuiltinUserEntity>;
