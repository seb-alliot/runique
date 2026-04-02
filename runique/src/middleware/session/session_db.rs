use async_trait::async_trait;
use sea_orm::{
    ActiveValue::Set, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter,
    entity::prelude::*, sea_query::OnConflict,
};
use std::sync::Arc;
use tower_sessions::{
    SessionStore,
    cookie::time::OffsetDateTime,
    session::{Id, Record},
    session_store,
};

// ─────────────────────────────────────────────────────────────────────────────
// Entité sea-orm — eihwaz_sessions
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, serde::Serialize, serde::Deserialize)]
#[sea_orm(table_name = "eihwaz_sessions")]
pub struct Model {
    /// ID de session tower-sessions (cookie, par appareil)
    #[sea_orm(primary_key, auto_increment = false)]
    pub session_id: String,

    /// ID stable par utilisateur — inchangé entre les logins, permet le multi-appareils
    pub session_id_user: Option<String>,

    /// FK → eihwaz_users.id — NULL pour les sessions anonymes
    pub user_id: Option<i32>,

    /// Appareil détecté (mobile, desktop, tablet…)
    pub device: Option<String>,

    /// Données de session sérialisées en JSON
    pub data: serde_json::Value,

    /// Date d'expiration
    pub expires_at: chrono::DateTime<chrono::Utc>,
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
// Helpers de conversion de dates
// ─────────────────────────────────────────────────────────────────────────────

fn offset_to_chrono(dt: OffsetDateTime) -> chrono::DateTime<chrono::Utc> {
    chrono::DateTime::from_timestamp(dt.unix_timestamp(), dt.nanosecond())
        .unwrap_or_else(chrono::Utc::now)
}

fn chrono_to_offset(dt: chrono::DateTime<chrono::Utc>) -> OffsetDateTime {
    OffsetDateTime::from_unix_timestamp(dt.timestamp())
        .unwrap_or_else(|_| OffsetDateTime::now_utc())
}

fn extract_user_id(data: &std::collections::HashMap<String, serde_json::Value>) -> Option<i32> {
    data.get(crate::utils::constante::SESSION_USER_ID_KEY)
        .and_then(serde_json::Value::as_i64)
        .map(|id| id as i32)
}

// ─────────────────────────────────────────────────────────────────────────────
// RuniqueSessionStore — implémente tower_sessions::SessionStore via DB
// ─────────────────────────────────────────────────────────────────────────────

/// Store de sessions DB pour tower-sessions.
///
/// Remplace `CleaningMemoryStore` pour une persistance réelle :
/// - Sessions survivent au redémarrage du serveur
/// - Multi-appareils via `session_id_user`
/// - RAM constante — rien en mémoire entre les requêtes
///
/// # Usage
/// ```rust,ignore
/// let store = RuniqueSessionStore::new(db.clone());
/// store.spawn_cleanup(Duration::from_secs(3600));
/// .session(SessionConfig {
///     session: SessionBackend::Custom(Arc::new(store)),
///     ..Default::default()
/// })
/// ```
#[derive(Clone, Debug)]
pub struct RuniqueSessionStore {
    db: Arc<DatabaseConnection>,
}

impl RuniqueSessionStore {
    pub fn new(db: Arc<DatabaseConnection>) -> Self {
        Self { db }
    }

    /// Spawne la tâche de nettoyage des sessions expirées.
    pub fn spawn_cleanup(&self, period: tokio::time::Duration) {
        let db = self.db.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(period);
            loop {
                interval.tick().await;
                let now = chrono::Utc::now();
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

    /// Invalide toutes les sessions d'un utilisateur (ex: changement de mot de passe).
    pub async fn invalidate_user_sessions(&self, user_id: i32) {
        let _ = Entity::delete_many()
            .filter(Column::UserId.eq(user_id))
            .exec(&*self.db)
            .await;
    }

    /// Récupère ou génère le `session_id_user` stable pour un utilisateur.
    async fn get_or_create_session_id_user(&self, user_id: i32) -> String {
        // Cherche un session_id_user existant pour cet utilisateur
        if let Ok(Some(existing)) = Entity::find()
            .filter(Column::UserId.eq(user_id))
            .filter(Column::SessionIdUser.is_not_null())
            .one(&*self.db)
            .await
        {
            if let Some(sid) = existing.session_id_user {
                return sid;
            }
        }
        // Aucun trouvé → génère un nouveau UUID stable
        uuid::Uuid::new_v4().to_string()
    }
}

#[async_trait]
impl SessionStore for RuniqueSessionStore {
    async fn create(&self, record: &mut Record) -> session_store::Result<()> {
        let session_id = record.id.to_string();
        let data = serde_json::to_value(&record.data)
            .map_err(|e| session_store::Error::Encode(e.to_string()))?;
        let expires_at = offset_to_chrono(record.expiry_date);
        let user_id = extract_user_id(&record.data);

        let session_id_user = if let Some(uid) = user_id {
            Some(self.get_or_create_session_id_user(uid).await)
        } else {
            None
        };

        let model = ActiveModel {
            session_id: Set(session_id),
            session_id_user: Set(session_id_user),
            user_id: Set(user_id),
            device: Set(None),
            data: Set(data),
            expires_at: Set(expires_at),
        };

        Entity::insert(model)
            .on_conflict(
                OnConflict::column(Column::SessionId)
                    .update_columns([
                        Column::Data,
                        Column::ExpiresAt,
                        Column::UserId,
                        Column::SessionIdUser,
                    ])
                    .to_owned(),
            )
            .exec(&*self.db)
            .await
            .map_err(|e| session_store::Error::Backend(e.to_string()))?;

        Ok(())
    }

    async fn save(&self, record: &Record) -> session_store::Result<()> {
        let session_id = record.id.to_string();
        let data = serde_json::to_value(&record.data)
            .map_err(|e| session_store::Error::Encode(e.to_string()))?;
        let expires_at = offset_to_chrono(record.expiry_date);
        let user_id = extract_user_id(&record.data);

        // Récupère le session_id_user existant si déjà en DB
        let existing = Entity::find_by_id(&session_id)
            .one(&*self.db)
            .await
            .ok()
            .flatten();

        let session_id_user = match existing {
            Some(ref m) if m.session_id_user.is_some() => m.session_id_user.clone(),
            _ => {
                if let Some(uid) = user_id {
                    Some(self.get_or_create_session_id_user(uid).await)
                } else {
                    None
                }
            }
        };

        let model = ActiveModel {
            session_id: Set(session_id),
            session_id_user: Set(session_id_user),
            user_id: Set(user_id),
            device: Set(existing.and_then(|m| m.device)),
            data: Set(data),
            expires_at: Set(expires_at),
        };

        Entity::insert(model)
            .on_conflict(
                OnConflict::column(Column::SessionId)
                    .update_columns([
                        Column::Data,
                        Column::ExpiresAt,
                        Column::UserId,
                        Column::SessionIdUser,
                    ])
                    .to_owned(),
            )
            .exec(&*self.db)
            .await
            .map_err(|e| session_store::Error::Backend(e.to_string()))?;

        Ok(())
    }

    async fn load(&self, session_id: &Id) -> session_store::Result<Option<Record>> {
        let id_str = session_id.to_string();
        let now = chrono::Utc::now();

        let model = Entity::find_by_id(&id_str)
            .filter(Column::ExpiresAt.gt(now))
            .one(&*self.db)
            .await
            .map_err(|e| session_store::Error::Backend(e.to_string()))?;

        let Some(m) = model else { return Ok(None) };

        let data: std::collections::HashMap<String, serde_json::Value> =
            serde_json::from_value(m.data)
                .map_err(|e| session_store::Error::Decode(e.to_string()))?;

        Ok(Some(Record {
            id: *session_id,
            data,
            expiry_date: chrono_to_offset(m.expires_at),
        }))
    }

    async fn delete(&self, session_id: &Id) -> session_store::Result<()> {
        Entity::delete_by_id(session_id.to_string())
            .exec(&*self.db)
            .await
            .map_err(|e| session_store::Error::Backend(e.to_string()))?;
        Ok(())
    }
}
