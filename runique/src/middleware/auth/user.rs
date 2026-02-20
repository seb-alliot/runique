// ═══════════════════════════════════════════════════════════════
// User built-in de Runique
// ═══════════════════════════════════════════════════════════════
//
// Prêt à l'emploi, aucune migration manuelle requise.
// Table : `runique_users` (préfixé pour éviter les conflits).
//
// Usage zéro config :
//   .with_admin(|a| a.auth(RuniqueAdminAuth::new()))
// ═══════════════════════════════════════════════════════════════

use sea_orm::{entity::prelude::*, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};

use crate::impl_objects;
pub use crate::middleware::auth::default_auth::UserEntity;
pub use crate::middleware::auth::user_trait::RuniqueUser;

// ─── Modèle SeaORM ───────────────────────────────────────────────────────────

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, serde::Serialize, serde::Deserialize)]
#[sea_orm(table_name = "eihwaz_users")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub username: String,
    pub email: String,
    /// Hash Argon2 — jamais le mot de passe en clair
    pub password: String,
    pub is_active: bool,
    pub is_staff: bool,
    pub is_superuser: bool,
    /// Format JSON : ["editor","moderator"] ou NULL
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
            .and_then(|r| serde_json::from_str(r).ok())
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
        Entity::find()
            .filter(Column::Username.eq(username))
            .one(db)
            .await
            .ok()
            .flatten()
    }
    async fn find_by_email(db: &DatabaseConnection, email: &str) -> Option<Self::Model> {
        Entity::find()
            .filter(Column::Email.eq(email))
            .one(db)
            .await
            .ok()
            .flatten()
    }
}

// ─── Alias pratique ───────────────────────────────────────────────────────────

/// Auth admin prête à l'emploi avec le User built-in.
///
/// ```rust,ignore
///         .with_admin(|a| {
///          a.with_registry(admin::admin_config())
///              .hot_reload(cfg!(debug_assertions))
///              .site_title("Administration")
///              .auth(RuniqueAdminAuth::new())
///              .routes(admins::admin("/admin"))
/// ```
pub type RuniqueAdminAuth =
    crate::middleware::auth::default_auth::DefaultAdminAuth<BuiltinUserEntity>;
