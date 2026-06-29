//! In-memory session store with automatic cleanup and watermarks to prevent leaks.
use std::{
    collections::HashMap,
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
};

#[cfg(feature = "orm")]
use super::session_db::RuniqueSessionStore;

use crate::utils::config::TraceResult;
use async_trait::async_trait;
use tokio::sync::Mutex;
use tower_sessions::{
    SessionStore,
    cookie::time::OffsetDateTime,
    session::{Id, Record},
    session_store::{self, ExpiredDeletion},
};

// ═══════════════════════════════════════════════════════════════
// Constants
// ═══════════════════════════════════════════════════════════════

/// Triggers proactive cleanup of expired anonymous sessions.
const LOW_WATERMARK_DEFAULT: usize = 128 * 1024 * 1024; // 128 MB

/// Triggers synchronous emergency cleanup + refuses new sessions if insufficient.
const HIGH_WATERMARK_DEFAULT: usize = 256 * 1024 * 1024; // 256 MB

/// Alerts in logs if a session record exceeds this size.
const MAX_SESSION_RECORD_SIZE: usize = 50 * 1024; // 50 KB

// ═══════════════════════════════════════════════════════════════
// Internal helpers
// ═══════════════════════════════════════════════════════════════

/// Estimated in-memory size of a record.
/// UUID (16b) + `expiry_date` (8b) + serialized JSON data.
fn estimate_size(record: &Record) -> usize {
    24usize.saturating_add(
        serde_json::to_string(&record.data)
            .map(|s| s.len())
            .unwrap_or(256),
    )
}

/// A session is protected if it belongs to an authenticated user
/// (`user_id` present) or if the dev set `session_active` with a future timestamp.
fn is_protected(record: &Record) -> bool {
    record.data.contains_key("user_id")
        || record
            .data
            .get("session_active")
            .and_then(serde_json::Value::as_i64)
            .is_some_and(|ts| ts > OffsetDateTime::now_utc().unix_timestamp())
}

// ═══════════════════════════════════════════════════════════════
// CleaningMemoryStore
// ═══════════════════════════════════════════════════════════════

/// `MemoryStore` with automatic purge and watermark protection.
///
/// # Behavior
///
/// **Timer (60s)**: purges all expired sessions (anonymous and authenticated).
///
/// **Low watermark**: asynchronous (non-blocking) purge of expired anonymous sessions.
///
/// **High watermark**: synchronous emergency purge.
///   - Phase 1: expired anonymous sessions
///   - Phase 2: all expired sessions (if still exceeded)
///   - If still exceeded → `Error::Backend` (503 for the client)
///
/// # Session Protection
///
/// Sessions with `user_id` or `session_active` (future timestamp) are never
/// deleted under pressure — only valueless anonymous sessions are sacrificed.
#[derive(Clone, Debug)]
pub struct CleaningMemoryStore {
    data: Arc<Mutex<HashMap<Id, Record>>>,
    size_bytes: Arc<AtomicUsize>,
    low_watermark: usize,
    high_watermark: usize,
    /// If `true`, any new connection invalidates existing sessions of the same user.
    exclusive_login: bool,
    /// DB store used to restore authenticated sessions after a restart (warm restart).
    #[cfg(feature = "orm")]
    db_fallback: Option<Arc<RuniqueSessionStore>>,
}

impl Default for CleaningMemoryStore {
    fn default() -> Self {
        Self {
            data: Arc::new(Mutex::new(HashMap::new())),
            size_bytes: Arc::new(AtomicUsize::new(0)),
            low_watermark: LOW_WATERMARK_DEFAULT,
            high_watermark: HIGH_WATERMARK_DEFAULT,
            exclusive_login: false,
            #[cfg(feature = "orm")]
            db_fallback: None,
        }
    }
}

impl CleaningMemoryStore {
    /// Configures memory watermarks.
    ///
    /// - `low`: triggers proactive cleanup (non-blocking)
    /// - `high`: triggers emergency cleanup + refuses if insufficient
    #[must_use]
    pub fn with_watermarks(mut self, low: usize, high: usize) -> Self {
        self.low_watermark = low;
        self.high_watermark = high;
        self
    }

    /// Enables or disables exclusive login.
    ///
    /// If `true`, any new connection automatically invalidates
    /// existing sessions of the same user — only one device connected at a time.
    #[must_use]
    pub fn with_exclusive_login(mut self, exclusive: bool) -> Self {
        self.exclusive_login = exclusive;
        self
    }

    /// Enables DB fallback: authenticated sessions are persisted and restored after a restart.
    #[cfg(feature = "orm")]
    #[must_use]
    pub fn with_db_fallback(mut self, db: Arc<RuniqueSessionStore>) -> Self {
        self.db_fallback = Some(db);
        self
    }

    /// Mirrors the authoritative in-memory record to the DB backup as a single full
    /// snapshot (cookie_id, user_id, expiry, data). The unique persistence path for
    /// both `create()` and `save()`: writing every field together makes a partial /
    /// stale backup (e.g. data updated but not expiry) impossible by construction.
    ///
    /// No-op for anonymous sessions (no user_id) or when DB fallback is disabled.
    /// A write failure is logged, never silently dropped — a silent backup failure
    /// breaks the "transparent UX after restart" promise without any signal.
    #[cfg(feature = "orm")]
    async fn persist_to_db(&self, record: &Record) {
        let Some(ref db) = self.db_fallback else {
            return;
        };
        let Some(user_id) = record
            .data
            .get(crate::utils::constante::session_key::session::SESSION_USER_ID_KEY)
            .and_then(|v| serde_json::from_value::<crate::utils::pk::Pk>(v.clone()).ok())
        else {
            return;
        };
        let expires_at = chrono::DateTime::from_timestamp(record.expiry_date.unix_timestamp(), 0)
            .map(|dt| dt.naive_utc())
            .unwrap_or_else(|| chrono::Utc::now().naive_utc());
        let data = serde_json::to_string(&record.data).trace(
            crate::utils::runique_log::get_log()
                .session
                .as_ref()
                .and_then(|s| s.store),
            "serialize session data for DB backup",
        );

        if let Err(e) = db
            .upsert_session(&record.id.to_string(), user_id, expires_at, data)
            .await
            && let Some(level) = crate::utils::runique_log::get_log()
                .session
                .as_ref()
                .and_then(|s| s.store)
        {
            crate::runique_log!(level, "session DB backup write failed: {e}");
        }
    }

    /// Current estimated size of the store in bytes.
    #[must_use]
    pub fn size_bytes(&self) -> usize {
        self.size_bytes.load(Ordering::Relaxed)
    }

    /// `true` if the store has reached its high watermark — a new session `create()`
    /// would be refused after the emergency purge. Exposed so request handlers (login)
    /// can fail **fast and clean** (503 + `Retry-After`) instead of letting tower's
    /// commit-time `create` error bubble up as a generic 500.
    #[must_use]
    pub fn is_saturated(&self) -> bool {
        self.size_bytes.load(Ordering::Relaxed) >= self.high_watermark
    }

    /// Invalidates all active sessions for a user.
    ///
    /// Used to implement exclusive login (only one device at a time).
    /// The current session (new login) should not yet contain `user_id`
    /// at the time of call — it is thus preserved.
    pub async fn invalidate_user_sessions(&self, user_id: crate::utils::pk::Pk) {
        let mut guard = self.data.lock().await;
        let mut freed = 0usize;
        #[allow(clippy::useless_conversion)]
        let uid_i64: i64 = user_id.into();
        let to_delete: Vec<Id> = guard
            .iter()
            .filter(|(_, r)| {
                r.data
                    .get(crate::utils::constante::session_key::session::SESSION_USER_ID_KEY)
                    .and_then(serde_json::Value::as_i64)
                    .is_some_and(|id| id == uid_i64)
            })
            .map(|(id, _)| *id)
            .collect();
        for id in to_delete {
            if let Some(r) = guard.remove(&id) {
                freed = freed.saturating_add(estimate_size(&r));
            }
        }
        if freed > 0 {
            self.size_bytes.fetch_sub(freed, Ordering::Relaxed);
        }
    }

    /// Spawns the Tokio task for periodic cleanup.
    pub fn spawn_cleanup(&self, period: tokio::time::Duration) {
        let store = self.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(period);
            loop {
                interval.tick().await;
                if let Err(e) = store.delete_expired().await
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

    /// Asynchronous purge of expired anonymous sessions (low watermark response).
    async fn purge_anonymous_expired(&self) {
        let now = OffsetDateTime::now_utc();
        let mut guard = self.data.lock().await;
        let mut freed = 0usize;

        let to_delete: Vec<Id> = guard
            .iter()
            .filter(|(_, r)| !is_protected(r) && r.expiry_date <= now)
            .map(|(id, _)| *id)
            .collect();

        for id in to_delete {
            if let Some(r) = guard.remove(&id) {
                freed = freed.saturating_add(estimate_size(&r));
            }
        }

        if freed > 0 {
            self.size_bytes.fetch_sub(freed, Ordering::Relaxed);
            if let Some(level) = crate::utils::runique_log::get_log()
                .session
                .as_ref()
                .and_then(|s| s.store)
            {
                crate::runique_log!(
                    level,
                    "Low watermark: {} bytes freed (expired anonymous sessions)",
                    freed
                );
            }
        }
    }
}

// ═══════════════════════════════════════════════════════════════
// SessionStore
// ═══════════════════════════════════════════════════════════════

#[async_trait]
impl SessionStore for CleaningMemoryStore {
    async fn create(&self, record: &mut Record) -> session_store::Result<()> {
        let current = self.size_bytes.load(Ordering::Relaxed);

        // Low watermark — non-blocking background cleanup
        if current >= self.low_watermark && current < self.high_watermark {
            let store = self.clone();
            tokio::spawn(async move {
                store.purge_anonymous_expired().await;
            });
        }

        let mut guard = self.data.lock().await;

        // High watermark — synchronous emergency cleanup under the lock
        if self.size_bytes.load(Ordering::Relaxed) >= self.high_watermark {
            let now = OffsetDateTime::now_utc();

            // Phase 1: expired anonymous sessions
            let mut freed = 0usize;
            let to_delete: Vec<Id> = guard
                .iter()
                .filter(|(_, r)| !is_protected(r) && r.expiry_date <= now)
                .map(|(id, _)| *id)
                .collect();
            for id in to_delete {
                if let Some(r) = guard.remove(&id) {
                    freed = freed.saturating_add(estimate_size(&r));
                }
            }
            self.size_bytes.fetch_sub(freed, Ordering::Relaxed);

            // Phase 2: all expired sessions (if still above)
            if self.size_bytes.load(Ordering::Relaxed) >= self.high_watermark {
                let mut freed2 = 0usize;
                let to_delete2: Vec<Id> = guard
                    .iter()
                    .filter(|(_, r)| r.expiry_date <= now)
                    .map(|(id, _)| *id)
                    .collect();
                for id in to_delete2 {
                    if let Some(r) = guard.remove(&id) {
                        freed2 = freed2.saturating_add(estimate_size(&r));
                    }
                }
                self.size_bytes.fetch_sub(freed2, Ordering::Relaxed);
                if let Some(level) = crate::utils::runique_log::get_log()
                    .session
                    .as_ref()
                    .and_then(|s| s.store)
                {
                    crate::runique_log!(
                        level,
                        "High watermark: {} + {} bytes freed in emergency",
                        freed,
                        freed2
                    );
                }
            }

            // Still above → refusal
            if self.size_bytes.load(Ordering::Relaxed) >= self.high_watermark {
                if let Some(level) = crate::utils::runique_log::get_log()
                    .session
                    .as_ref()
                    .and_then(|s| s.store)
                {
                    crate::runique_log!(
                        level,
                        "Session store saturated ({} bytes), new session refused",
                        self.size_bytes()
                    );
                }
                return Err(session_store::Error::Backend(
                    "session store capacity exceeded".into(),
                ));
            }
        }

        // Insertion
        while guard.contains_key(&record.id) {
            record.id = Id::default();
        }

        let size = estimate_size(record);
        if size > MAX_SESSION_RECORD_SIZE
            && let Some(level) = crate::utils::runique_log::get_log()
                .session
                .as_ref()
                .and_then(|s| s.store)
        {
            crate::runique_log!(
                level,
                "Large session record ({} bytes) — avoid storing files or images in session",
                size
            );
        }

        guard.insert(record.id, record.clone());
        self.size_bytes.fetch_add(size, Ordering::Relaxed);

        // Release the lock before the async DB write (cycle_id path: tower-sessions calls
        // create() instead of save() for a recycled session, so we persist here too).
        drop(guard);

        // Mirror the authoritative in-memory record to the DB backup.
        #[cfg(feature = "orm")]
        self.persist_to_db(record).await;

        Ok(())
    }

    async fn save(&self, record: &Record) -> session_store::Result<()> {
        let mut guard = self.data.lock().await;
        let old_size = guard.get(&record.id).map_or(0, estimate_size);
        let new_size = estimate_size(record);

        // Exclusive login: if user_id appears for the first time on this session,
        // invalidate all other sessions for the same user.
        #[cfg(feature = "orm")]
        let mut exclusive_db_invalidate: Option<(
            Arc<RuniqueSessionStore>,
            crate::utils::pk::Pk,
            String,
        )> = None;

        if self.exclusive_login {
            let had_user = guard
                .get(&record.id)
                .and_then(|r| {
                    r.data
                        .get(crate::utils::constante::session_key::session::SESSION_USER_ID_KEY)
                })
                .is_some();
            if let Some(user_id) = record
                .data
                .get(crate::utils::constante::session_key::session::SESSION_USER_ID_KEY)
                .and_then(|v| serde_json::from_value::<crate::utils::pk::Pk>(v.clone()).ok())
                && !had_user
            {
                let mut freed = 0usize;
                let to_delete: Vec<Id> = guard
                        .iter()
                        .filter(|(id, r)| {
                            **id != record.id
                                && r.data
                                    .get(crate::utils::constante::session_key::session::SESSION_USER_ID_KEY)
                                    .and_then(|v| serde_json::from_value::<crate::utils::pk::Pk>(v.clone()).ok())
                                    .is_some_and(|id| id == user_id)
                        })
                        .map(|(id, _)| *id)
                        .collect();
                for id in to_delete {
                    if let Some(r) = guard.remove(&id) {
                        freed = freed.saturating_add(estimate_size(&r));
                    }
                }
                if freed > 0 {
                    self.size_bytes.fetch_sub(freed, Ordering::Relaxed);
                    if let Some(level) = crate::utils::runique_log::get_log()
                        .session
                        .as_ref()
                        .and_then(|s| s.exclusive_login)
                    {
                        crate::runique_log!(
                            level,
                            user_id = user_id,
                            "exclusive_login: {} session(s) invalidated for user {}",
                            freed,
                            user_id
                        );
                    }
                }

                // Collect params for DB-level invalidation (executed after lock release).
                #[cfg(feature = "orm")]
                if let Some(ref db) = self.db_fallback {
                    exclusive_db_invalidate = Some((db.clone(), user_id, record.id.to_string()));
                }
            }
        }

        if new_size > MAX_SESSION_RECORD_SIZE
            && let Some(level) = crate::utils::runique_log::get_log()
                .session
                .as_ref()
                .and_then(|s| s.store)
        {
            crate::runique_log!(
                level,
                "Large session record ({} bytes) — avoid storing files or images in session",
                new_size
            );
        }

        guard.insert(record.id, record.clone());

        if new_size >= old_size {
            self.size_bytes
                .fetch_add(new_size.saturating_sub(old_size), Ordering::Relaxed);
        } else {
            self.size_bytes
                .fetch_sub(old_size.saturating_sub(new_size), Ordering::Relaxed);
        }

        // Release the lock before the async DB writes to avoid holding it during I/O.
        drop(guard);

        #[cfg(feature = "orm")]
        if let Some((db, user_id, cookie_id)) = exclusive_db_invalidate {
            db.invalidate_other_sessions(user_id, &cookie_id)
                .await
                .trace(
                    crate::utils::runique_log::get_log()
                        .session
                        .as_ref()
                        .and_then(|s| s.exclusive_login),
                    "invalidate other sessions (exclusive login)",
                );
        }

        // Mirror the authoritative in-memory record to the DB backup (full snapshot,
        // refreshed expiry included) so it survives restarts.
        #[cfg(feature = "orm")]
        self.persist_to_db(record).await;

        Ok(())
    }

    async fn load(&self, session_id: &Id) -> session_store::Result<Option<Record>> {
        let in_memory = self
            .data
            .lock()
            .await
            .get(session_id)
            .filter(|r| r.expiry_date > OffsetDateTime::now_utc())
            .cloned();

        if in_memory.is_some() {
            return Ok(in_memory);
        }

        // DB fallback: restores authenticated sessions after a server restart.
        #[cfg(feature = "orm")]
        if let Some(ref db) = self.db_fallback {
            let cookie_id = session_id.to_string();
            // A DB error here (schema drift, missing table) must NOT be swallowed:
            // it silently kills the fallback and surfaces only as "record not found".
            match db.find_by_cookie_id(&cookie_id).await {
                Ok(Some(model)) => {
                    if let Some(data_str) = &model.session_data
                        && let Ok(data) = serde_json::from_str(data_str)
                    {
                        let expiry = OffsetDateTime::from_unix_timestamp(
                            model.expires_at.and_utc().timestamp(),
                        )
                        .unwrap_or_else(|_| OffsetDateTime::now_utc());
                        let record = Record {
                            id: *session_id,
                            data,
                            expiry_date: expiry,
                        };
                        // Warm the memory cache
                        let mut guard = self.data.lock().await;
                        guard.insert(record.id, record.clone());
                        self.size_bytes
                            .fetch_add(estimate_size(&record), Ordering::Relaxed);
                        return Ok(Some(record));
                    }
                }
                Ok(None) => {}
                Err(e) => {
                    if let Some(level) = crate::utils::runique_log::get_log()
                        .session
                        .as_ref()
                        .and_then(|s| s.store)
                    {
                        crate::runique_log!(
                            level,
                            "session DB fallback load failed for cookie_id lookup: {e}"
                        );
                    }
                }
            }
        }

        Ok(None)
    }

    async fn delete(&self, session_id: &Id) -> session_store::Result<()> {
        let mut guard = self.data.lock().await;
        if let Some(r) = guard.remove(session_id) {
            self.size_bytes
                .fetch_sub(estimate_size(&r), Ordering::Relaxed);
        }
        drop(guard);

        // Remove from DB too (covers cycle_id() old-ID cleanup and explicit session.delete()).
        // logout() already calls store.delete() before session.delete(), so this is idempotent.
        #[cfg(feature = "orm")]
        if let Some(ref db) = self.db_fallback {
            db.delete(&session_id.to_string()).await.trace(
                crate::utils::runique_log::get_log()
                    .session
                    .as_ref()
                    .and_then(|s| s.store),
                "delete session from DB",
            );
        }

        Ok(())
    }
}

// ═══════════════════════════════════════════════════════════════
// ExpiredDeletion
// ═══════════════════════════════════════════════════════════════

#[async_trait]
impl ExpiredDeletion for CleaningMemoryStore {
    async fn delete_expired(&self) -> session_store::Result<()> {
        let now = OffsetDateTime::now_utc();
        let mut guard = self.data.lock().await;
        let mut freed = 0usize;

        guard.retain(|_, r| {
            if r.expiry_date <= now {
                freed = freed.saturating_add(estimate_size(r));
                false
            } else {
                true
            }
        });

        if freed > 0 {
            self.size_bytes.fetch_sub(freed, Ordering::Relaxed);
        }
        Ok(())
    }
}
