//! Persistent session store in the database (table `eihwaz_sessions`).
use sea_orm::{
    ActiveValue::Set, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, entity::prelude::*,
};
use sea_query::OnConflict;
use std::sync::Arc;

// ─────────────────────────────────────────────────────────────────────────────
// SeaORM Entity — eihwaz_sessions
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, serde::Serialize, serde::Deserialize)]
#[sea_orm(table_name = "eihwaz_sessions")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,

    /// tower-sessions session ID (browser cookie) — unique per device
    #[sea_orm(unique)]
    pub cookie_id: String,

    /// FK → eihwaz_users.id
    pub user_id: crate::utils::pk::Pk,

    /// Stable identifier per login/device
    pub session_id: String,

    /// Serialized session data (JSON)
    pub session_data: Option<String>,

    /// Expiration date
    pub expires_at: chrono::NaiveDateTime,
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
}

impl Related<crate::auth::user::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::User.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

// ─────────────────────────────────────────────────────────────────────────────
// RuniqueSessionStore — explicit DB layer (login / logout / multi-device)
// ─────────────────────────────────────────────────────────────────────────────

/// Management of authenticated sessions in DB.
///
/// Does not implement `tower_sessions::SessionStore` — tower-sessions
/// continues to use the memory store for CSRF and anonymous sessions.
///
/// Called explicitly at login and logout.
#[derive(Clone, Debug)]
pub struct RuniqueSessionStore {
    db: Arc<DatabaseConnection>,
}

impl RuniqueSessionStore {
    pub fn new(db: Arc<DatabaseConnection>) -> Self {
        Self { db }
    }

    /// Creates a DB entry for an authenticated session.
    pub async fn create(
        &self,
        cookie_id: &str,
        user_id: crate::utils::pk::Pk,
        session_id: &str,
        expires_at: chrono::NaiveDateTime,
    ) -> Result<(), DbErr> {
        // Zero-trust : un cookie_id vide casserait la contrainte unique dès la 2ᵉ ligne.
        // On rejette tôt plutôt que de laisser remonter une erreur SQL opaque.
        if cookie_id.is_empty() {
            return Err(DbErr::Custom(
                "create session: empty cookie_id rejected".to_string(),
            ));
        }
        let model = ActiveModel {
            cookie_id: Set(cookie_id.to_string()),
            user_id: Set(user_id),
            session_id: Set(session_id.to_string()),
            session_data: Set(None),
            expires_at: Set(expires_at),
            ..Default::default()
        };
        // Upsert atomique, portable (Postgres ON CONFLICT / MySQL ON DUPLICATE KEY /
        // SQLite ON CONFLICT — généré par SeaORM selon le backend).
        // Au restart, la ligne d'avant survit en DB alors que la mémoire est vide ; le 1er
        // login post-restart réutilise le même cookie_id (cycle_id() n'est pas appelé sans
        // élévation de privilège) → sans ON CONFLICT, l'INSERT collisionne.
        // On ne met à jour QUE user_id + expires_at : session_id reste stable (même cookie =
        // même device) et session_data appartient à upsert_session (ne pas l'écraser à None,
        // sinon un login post-restart effacerait les données restaurées).
        // exec_without_returning : pas de clause RETURNING → évite l'erreur RecordNotInserted
        // que certains backends renvoient quand le conflit déclenche un UPDATE sans INSERT.
        Entity::insert(model)
            .on_conflict(
                OnConflict::column(Column::CookieId)
                    .update_columns([Column::UserId, Column::ExpiresAt])
                    .to_owned(),
            )
            .exec_without_returning(&*self.db)
            .await?;
        Ok(())
    }

    /// Deletes the DB session corresponding to the cookie_id (logout).
    pub async fn delete(&self, cookie_id: &str) -> Result<(), DbErr> {
        Entity::delete_many()
            .filter(Column::CookieId.eq(cookie_id))
            .exec(&*self.db)
            .await?;
        Ok(())
    }

    /// Invalidates all sessions of a user except the current one (exclusive login).
    pub async fn invalidate_other_sessions(
        &self,
        user_id: crate::utils::pk::Pk,
        current_cookie_id: &str,
    ) -> Result<(), DbErr> {
        Entity::delete_many()
            .filter(Column::UserId.eq(user_id))
            .filter(Column::CookieId.ne(current_cookie_id))
            .exec(&*self.db)
            .await?;
        Ok(())
    }

    /// Invalidates all sessions of a user (password change, etc.).
    pub async fn invalidate_all(&self, user_id: i32) -> Result<(), DbErr> {
        Entity::delete_many()
            .filter(Column::UserId.eq(user_id))
            .exec(&*self.db)
            .await?;
        Ok(())
    }

    /// Returns the active session matching `cookie_id`, or `None` if absent/expired.
    pub async fn find_by_cookie_id(&self, cookie_id: &str) -> Result<Option<Model>, DbErr> {
        let now = chrono::Utc::now().naive_utc();
        Entity::find()
            .filter(Column::CookieId.eq(cookie_id))
            .filter(Column::ExpiresAt.gt(now))
            .one(&*self.db)
            .await
    }

    /// Persists serialized session data for an existing row (no-op if not found).
    /// Creates or updates a session entry (upsert).
    ///
    /// Used by `CleaningMemoryStore::create()` at response commit: the new cookie_id
    /// may not have a DB record yet (e.g. after `cycle_id()`), so a plain UPDATE
    /// would silently no-op. This method inserts when the record is absent.
    pub async fn upsert_session(
        &self,
        cookie_id: &str,
        user_id: crate::utils::pk::Pk,
        expires_at: chrono::NaiveDateTime,
        data: Option<String>,
    ) -> Result<(), DbErr> {
        if cookie_id.is_empty() {
            return Err(DbErr::Custom(
                "upsert session: empty cookie_id rejected".to_string(),
            ));
        }
        let model = ActiveModel {
            cookie_id: Set(cookie_id.to_string()),
            user_id: Set(user_id),
            session_id: Set(uuid::Uuid::new_v4().to_string()),
            session_data: Set(data),
            expires_at: Set(expires_at),
            ..Default::default()
        };
        // Upsert atomique cross-DB : remplace l'ancien UPDATE-puis-INSERT qui avait une
        // fenêtre de race (deux requêtes concurrentes voyaient "absent" puis collisionnaient).
        // On ne touche pas user_id/session_id sur conflit : ce chemin ne fait que sauvegarder
        // le payload (session_data) et prolonger le TTL d'une session existante.
        Entity::insert(model)
            .on_conflict(
                OnConflict::column(Column::CookieId)
                    .update_columns([Column::SessionData, Column::ExpiresAt])
                    .to_owned(),
            )
            .exec_without_returning(&*self.db)
            .await?;
        Ok(())
    }

    /// Returns all active sessions for a user.
    pub async fn find_by_user(&self, user_id: i32) -> Result<Vec<Model>, DbErr> {
        let now = chrono::Utc::now().naive_utc();
        Entity::find()
            .filter(Column::UserId.eq(user_id))
            .filter(Column::ExpiresAt.gt(now))
            .all(&*self.db)
            .await
    }

    /// Deletes expired sessions (should be called periodically).
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
                    && let Some(level) = crate::utils::runique_log::get_log()
                        .session
                        .as_ref()
                        .and_then(|s| s.store)
                {
                    crate::runique_log!(level, "session cleanup error: {e}");
                }
            }
        });
    }
}
