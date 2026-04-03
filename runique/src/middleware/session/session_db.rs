use sea_orm::{
    ActiveValue::Set, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, entity::prelude::*,
};
use std::sync::Arc;

// ─────────────────────────────────────────────────────────────────────────────
// Entité SeaORM — eihwaz_sessions
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, serde::Serialize, serde::Deserialize)]
#[sea_orm(table_name = "eihwaz_sessions")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,

    /// ID de session tower-sessions (cookie navigateur) — unique par appareil
    #[sea_orm(unique)]
    pub cookie_id: String,

    /// FK → eihwaz_users.id
    pub user_id: i32,

    /// Identifiant stable par login/appareil
    pub session_id: String,

    /// Données de session sérialisées (JSON)
    pub session_data: Option<String>,

    /// Date d'expiration
    pub expires_at: chrono::NaiveDateTime,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "crate::middleware::auth::user::Entity",
        from = "Column::UserId",
        to = "crate::middleware::auth::user::Column::Id",
        on_delete = "Cascade"
    )]
    User,
}

impl Related<crate::middleware::auth::user::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::User.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

// ─────────────────────────────────────────────────────────────────────────────
// RuniqueSessionStore — couche DB explicite (login / logout / multi-appareils)
// ─────────────────────────────────────────────────────────────────────────────

/// Gestion des sessions authentifiées en DB.
///
/// N'implémente pas `tower_sessions::SessionStore` — tower-sessions
/// continue d'utiliser le store mémoire pour le CSRF et les sessions anonymes.
///
/// Appelé explicitement au login et au logout.
#[derive(Clone, Debug)]
pub struct RuniqueSessionStore {
    db: Arc<DatabaseConnection>,
}

impl RuniqueSessionStore {
    pub fn new(db: Arc<DatabaseConnection>) -> Self {
        Self { db }
    }

    /// Crée une entrée DB pour une session authentifiée.
    pub async fn create(
        &self,
        cookie_id: &str,
        user_id: i32,
        session_id: &str,
        expires_at: chrono::NaiveDateTime,
    ) -> Result<(), DbErr> {
        let model = ActiveModel {
            cookie_id: Set(cookie_id.to_string()),
            user_id: Set(user_id),
            session_id: Set(session_id.to_string()),
            session_data: Set(None),
            expires_at: Set(expires_at),
            ..Default::default()
        };
        Entity::insert(model).exec(&*self.db).await?;
        Ok(())
    }

    /// Supprime la session DB correspondant au cookie_id (logout).
    pub async fn delete(&self, cookie_id: &str) -> Result<(), DbErr> {
        Entity::delete_many()
            .filter(Column::CookieId.eq(cookie_id))
            .exec(&*self.db)
            .await?;
        Ok(())
    }

    /// Invalide toutes les sessions d'un utilisateur sauf celle en cours (exclusive login).
    pub async fn invalidate_other_sessions(
        &self,
        user_id: i32,
        current_cookie_id: &str,
    ) -> Result<(), DbErr> {
        Entity::delete_many()
            .filter(Column::UserId.eq(user_id))
            .filter(Column::CookieId.ne(current_cookie_id))
            .exec(&*self.db)
            .await?;
        Ok(())
    }

    /// Invalide toutes les sessions d'un utilisateur (changement de mot de passe, etc.).
    pub async fn invalidate_all(&self, user_id: i32) -> Result<(), DbErr> {
        Entity::delete_many()
            .filter(Column::UserId.eq(user_id))
            .exec(&*self.db)
            .await?;
        Ok(())
    }

    /// Retourne toutes les sessions actives d'un utilisateur.
    pub async fn find_by_user(&self, user_id: i32) -> Result<Vec<Model>, DbErr> {
        let now = chrono::Utc::now().naive_utc();
        Entity::find()
            .filter(Column::UserId.eq(user_id))
            .filter(Column::ExpiresAt.gt(now))
            .all(&*self.db)
            .await
    }

    /// Supprime les sessions expirées (à appeler périodiquement).
    pub fn spawn_cleanup(&self, period: tokio::time::Duration) {
        let db = self.db.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(period);
            loop {
                interval.tick().await;
                let now = chrono::Utc::now().naive_utc();
                if let Err(e) = Entity::delete_many()
                    .filter(Column::ExpiresAt.lt(now))
                    .exec(&*db)
                    .await
                {
                    if let Some(level) = crate::utils::runique_log::get_log().session {
                        crate::runique_log!(level, "session cleanup error: {e}");
                    }
                }
            }
        });
    }
}
