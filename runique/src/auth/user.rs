//! Runique's built-in user entity (table `eihwaz_users`).
pub use crate::auth::{session::UserEntity, user_trait::RuniqueUser};
use crate::utils::pk::Pk;
use crate::{impl_objects, search};
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, DatabaseConnection, EntityTrait, entity::prelude::*,
};

// ─── SeaORM Model ───────────────────────────────────────────────────────────

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, serde::Serialize, serde::Deserialize)]
#[sea_orm(table_name = "eihwaz_users")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: Pk,
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
    #[sea_orm(has_many = "crate::admin::permissions::users_groupes::Entity")]
    UsersGroupes,
    #[sea_orm(has_many = "crate::middleware::session::session_db::Entity")]
    Sessions,
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
    fn user_id(&self) -> Pk {
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

    async fn find_by_id(db: &DatabaseConnection, id: Pk) -> Option<Self::Model> {
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

// ─── Handy Alias ───────────────────────────────────────────────────────────
pub type RuniqueAdminAuth = crate::auth::session::DefaultAdminAuth<BuiltinUserEntity>;

// ─── Form Schema ────────────────────────────────────────────────────────

/// Returns the `ModelSchema` of the `eihwaz_users` table.
/// Used by `#[form(schema = runique_users)]` — no need to declare the entity locally.
pub fn schema() -> crate::migration::schema::ModelSchema {
    #[cfg(feature = "big-pk")]
    let pk = crate::migration::PrimaryKeyDef::new("id")
        .i64()
        .auto_increment();
    #[cfg(not(feature = "big-pk"))]
    let pk = crate::migration::PrimaryKeyDef::new("id")
        .i32()
        .auto_increment();

    crate::migration::ModelSchema::new("EihwazUsers")
        .table_name("eihwaz_users")
        .primary_key(pk)
        .column(
            crate::migration::ColumnDef::new("username")
                .varchar(150)
                .required()
                .unique(),
        )
        .column(
            crate::migration::ColumnDef::new("email")
                .varchar(254)
                .required()
                .unique(),
        )
        .column(
            crate::migration::ColumnDef::new("password")
                .string()
                .required(),
        )
        .column(
            crate::migration::ColumnDef::new("is_active")
                .boolean()
                .required(),
        )
        .column(
            crate::migration::ColumnDef::new("is_staff")
                .boolean()
                .required(),
        )
        .column(
            crate::migration::ColumnDef::new("is_superuser")
                .boolean()
                .required(),
        )
        .column(
            crate::migration::ColumnDef::new("created_at")
                .datetime()
                .nullable(),
        )
        .column(
            crate::migration::ColumnDef::new("updated_at")
                .datetime()
                .nullable(),
        )
        .build()
        .unwrap()
}
