//! Persistent session store in the database (table `eihwaz_sessions`).
use crate::utils::aliases::ADb;
use crate::utils::crypto::session_cipher::SessionKey;
use sea_orm::{ActiveValue::Set, ColumnTrait, EntityTrait, QueryFilter, entity::prelude::*};
use sea_query::OnConflict;
use sha2::{Digest, Sha256};

fn hash_cookie_id(cookie_id: &str) -> String {
    hex::encode(Sha256::digest(cookie_id.as_bytes()))
}

// ─────────────────────────────────────────────────────────────────────────────
// SeaORM Entity — eihwaz_sessions
// ─────────────────────────────────────────────────────────────────────────────

/// SeaORM model for `eihwaz_sessions`: one row per authenticated session,
/// keyed by the tower-sessions cookie id.
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, serde::Serialize, serde::Deserialize)]
#[sea_orm(table_name = "eihwaz_sessions")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,

    /// SHA-256 hash of the tower-sessions cookie ID — unique per device.
    #[sea_orm(unique)]
    pub cookie_id: String,

    /// FK → eihwaz_users.id
    pub user_id: crate::utils::pk::Pk,

    /// Stable identifier per login/device
    pub session_id: String,

    /// Serialized session data (JSON), encrypted: read it with
    /// [`RuniqueSessionStore::open_session_data`].
    pub session_data: Option<String>,

    /// Expiration date
    pub expires_at: chrono::NaiveDateTime,
}

/// SeaORM relations for `eihwaz_sessions`.
#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    /// Belongs to the owning `eihwaz_users` row; deleting the user cascades
    /// to its sessions.
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
    db: ADb,
    key: SessionKey,
}

impl RuniqueSessionStore {
    /// Creates a store bound to the given database connection. `secret_key`
    /// encrypts the session data: changing it makes the stored sessions
    /// unreadable, and their users sign in again.
    pub fn new(db: ADb, secret_key: &str) -> Self {
        Self {
            db,
            key: SessionKey::derive(secret_key),
        }
    }

    /// Refuses to write rather than store a payload in clear.
    fn seal_data(&self, cookie_hash: &str, data: Option<String>) -> Result<Option<String>, DbErr> {
        data.map(|plain| {
            self.key
                .seal(&plain, cookie_hash)
                .ok_or_else(|| DbErr::Custom("session data could not be encrypted".to_string()))
        })
        .transpose()
    }

    /// The decrypted data of a row given by [`find_by_cookie_id`](Self::find_by_cookie_id).
    /// `None` when it has none or can't be read (another `SECRET_KEY`, an altered
    /// value, a value in clear from before 4.0, a payload from another row).
    #[must_use]
    pub fn open_session_data(&self, model: &Model) -> Option<String> {
        let sealed = model.session_data.as_deref()?;
        let opened = self.key.open(sealed, &hash_cookie_id(&model.cookie_id));
        if opened.is_none()
            && let Some(level) = crate::utils::runique_log::log_level!(session.store)
        {
            crate::runique_log!(level, "stored session data could not be decrypted");
        }
        opened
    }

    /// Creates a DB entry for an authenticated session.
    pub async fn create(
        &self,
        cookie_id: &str,
        user_id: crate::utils::pk::Pk,
        session_id: &str,
        expires_at: chrono::NaiveDateTime,
    ) -> Result<(), DbErr> {
        // Zero-trust: an empty cookie_id would break the unique constraint on the second row.
        // Reject early rather than let an opaque SQL error bubble up.
        if cookie_id.is_empty() {
            return Err(DbErr::Custom(
                "create session: empty cookie_id rejected".to_string(),
            ));
        }
        let model = ActiveModel {
            cookie_id: Set(hash_cookie_id(cookie_id)),
            user_id: Set(user_id),
            session_id: Set(session_id.to_string()),
            session_data: Set(None),
            expires_at: Set(expires_at),
            ..Default::default()
        };
        // Atomic, portable upsert (Postgres ON CONFLICT / MySQL ON DUPLICATE KEY /
        // SQLite ON CONFLICT — generated by SeaORM for the backend in use).
        // On restart, the previous row survives in the DB while memory is empty; the first
        // login after restart reuses the same cookie_id (cycle_id() is not called without
        // a privilege escalation) → without ON CONFLICT, the INSERT collides.
        // Only user_id + expires_at are updated: session_id stays stable (same cookie =
        // same device) and session_data belongs to upsert_session (do not overwrite it with
        // None, or a login after restart would wipe the restored data).
        // exec_without_returning: no RETURNING clause → avoids the RecordNotInserted error
        // some backends return when the conflict triggers an UPDATE instead of an INSERT.
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
            .filter(Column::CookieId.eq(hash_cookie_id(cookie_id)))
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
            .filter(Column::CookieId.ne(hash_cookie_id(current_cookie_id)))
            .exec(&*self.db)
            .await?;
        Ok(())
    }

    /// Invalidates all sessions of a user (password change, etc.).
    pub async fn invalidate_all(&self, user_id: crate::utils::pk::Pk) -> Result<(), DbErr> {
        Entity::delete_many()
            .filter(Column::UserId.eq(user_id))
            .exec(&*self.db)
            .await?;
        Ok(())
    }

    /// Returns the active session matching `cookie_id`, or `None` if absent/expired.
    pub async fn find_by_cookie_id(&self, cookie_id: &str) -> Result<Option<Model>, DbErr> {
        let now = chrono::Utc::now().naive_utc();
        let mut model = Entity::find()
            .filter(Column::CookieId.eq(hash_cookie_id(cookie_id)))
            .filter(Column::ExpiresAt.gt(now))
            .one(&*self.db)
            .await?;
        if let Some(model) = &mut model {
            model.cookie_id = cookie_id.to_string();
        }
        Ok(model)
    }

    /// Deletes, inside the caller's transaction, every session of `user_id`
    /// and the one behind `cookie_id`, whoever it belongs to: a password reset
    /// signs the account out everywhere, and the browser it ran in.
    pub(crate) async fn revoke_in_transaction(
        txn: &sea_orm::DatabaseTransaction,
        user_id: crate::utils::pk::Pk,
        cookie_id: Option<&str>,
    ) -> Result<(), DbErr> {
        let mut sessions = sea_orm::Condition::any().add(Column::UserId.eq(user_id));
        if let Some(cookie_id) = cookie_id {
            sessions = sessions.add(Column::CookieId.eq(hash_cookie_id(cookie_id)));
        }
        Entity::delete_many().filter(sessions).exec(txn).await?;
        Ok(())
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
        let cookie_hash = hash_cookie_id(cookie_id);
        let data = self.seal_data(&cookie_hash, data)?;
        let model = ActiveModel {
            cookie_id: Set(cookie_hash),
            user_id: Set(user_id),
            session_id: Set(uuid::Uuid::new_v4().to_string()),
            session_data: Set(data),
            expires_at: Set(expires_at),
            ..Default::default()
        };
        // Atomic cross-DB upsert: replaces the former UPDATE-then-INSERT, which had a
        // race window (two concurrent requests both saw "absent" and then collided).
        // user_id/session_id are left alone on conflict: this path only saves
        // the payload (session_data) and extends the TTL of an existing session.
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

    /// Updates a persisted session snapshot without creating a missing row.
    ///
    /// A zero-row update means the session was removed (for example by logout)
    /// while a request still held a stale in-memory copy.
    pub async fn update_session(
        &self,
        cookie_id: &str,
        user_id: crate::utils::pk::Pk,
        expires_at: chrono::NaiveDateTime,
        data: Option<String>,
    ) -> Result<bool, DbErr> {
        let cookie_hash = hash_cookie_id(cookie_id);
        let data = self.seal_data(&cookie_hash, data)?;
        let result = Entity::update_many()
            .col_expr(Column::SessionData, sea_orm::sea_query::Expr::value(data))
            .col_expr(
                Column::ExpiresAt,
                sea_orm::sea_query::Expr::value(expires_at),
            )
            .filter(Column::CookieId.eq(cookie_hash))
            .filter(Column::UserId.eq(user_id))
            .exec(&*self.db)
            .await?;
        Ok(result.rows_affected > 0)
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
                    && let Some(level) = crate::utils::runique_log::log_level!(session.cleanup)
                {
                    crate::runique_log!(level, "session cleanup error: {e}");
                }
            }
        });
    }
}
