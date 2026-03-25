// ═══════════════════════════════════════════════════════════════
// User built-in de Runique
// ═══════════════════════════════════════════════════════════════

pub use crate::middleware::auth::{default_auth::UserEntity, user_trait::RuniqueUser};
use crate::{impl_objects, search};
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, DatabaseConnection, EntityTrait, entity::prelude::*,
};

// ─── Modèle SeaORM ───────────────────────────────────────────────────────────

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, serde::Serialize, serde::Deserialize)]
#[sea_orm(table_name = "eihwaz_users")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub username: String,
    pub email: String,
    pub password: String,
    pub is_active: bool,
    pub is_staff: bool,
    pub is_superuser: bool,
    pub roles: Option<String>,
    pub created_at: Option<chrono::NaiveDateTime>,
    pub updated_at: Option<chrono::NaiveDateTime>,
}

impl_objects!(Entity);

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

// ─── Méthodes utilitaires ────────────────────────────────────────────────────
impl Model {
    pub fn get_roles(&self) -> Vec<String> {
        self.roles
            .as_deref()
            .map(|r| {
                serde_json::from_str(r).unwrap_or_else(|_| {
                    r.split(',')
                        .map(|s| s.trim().to_string())
                        .filter(|s| !s.is_empty())
                        .collect()
                })
            })
            .unwrap_or_default()
    }

    pub fn set_roles(roles: Vec<String>) -> Option<String> {
        if roles.is_empty() {
            None
        } else {
            serde_json::to_string(&roles).ok()
        }
    }
}

// ─── RuniqueUser ─────────────────────────────────────────────────────────────
impl RuniqueUser for Model {
    fn user_id(&self) -> i32 {
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
    fn roles(&self) -> Vec<String> {
        self.get_roles()
    }
}

// ─── UserEntity ──────────────────────────────────────────────────────────────
pub struct BuiltinUserEntity;

#[async_trait::async_trait]
impl UserEntity for BuiltinUserEntity {
    type Model = Model;

    async fn find_by_username(db: &DatabaseConnection, username: &str) -> Option<Self::Model> {
        search!(Entity => +Username = username)
            .first(db)
            .await
            .ok()
            .flatten()
    }

    async fn find_by_email(db: &DatabaseConnection, email: &str) -> Option<Self::Model> {
        search!(Entity => +Email = email)
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
        let user = search!(Entity => +Email = email)
            .first(db)
            .await?
            .ok_or(sea_orm::DbErr::RecordNotFound("User not found".into()))?;

        let mut active: ActiveModel = user.into();
        active.password = Set(new_hash.to_string());
        active.update(db).await?;
        Ok(())
    }
}

// ─── Alias pratique ───────────────────────────────────────────────────────────
pub type RuniqueAdminAuth =
    crate::middleware::auth::default_auth::DefaultAdminAuth<BuiltinUserEntity>;
