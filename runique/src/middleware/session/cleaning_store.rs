//! In-memory session store with automatic cleanup and watermarks to prevent leaks.
use std::{
    collections::HashMap,
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
};

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
}

impl Default for CleaningMemoryStore {
    fn default() -> Self {
        Self {
            data: Arc::new(Mutex::new(HashMap::new())),
            size_bytes: Arc::new(AtomicUsize::new(0)),
            low_watermark: LOW_WATERMARK_DEFAULT,
            high_watermark: HIGH_WATERMARK_DEFAULT,
            exclusive_login: false,
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

    /// Current estimated size of the store in bytes.
    #[must_use]
    pub fn size_bytes(&self) -> usize {
        self.size_bytes.load(Ordering::Relaxed)
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
                if let Err(e) = store.delete_expired().await {
                    if let Some(level) = crate::utils::runique_log::get_log().session {
                        crate::runique_log!(level, "session cleanup error: {e}");
                    }
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
            if let Some(level) = crate::utils::runique_log::get_log().session {
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
                if let Some(level) = crate::utils::runique_log::get_log().session {
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
                if let Some(level) = crate::utils::runique_log::get_log().session {
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
        if size > MAX_SESSION_RECORD_SIZE {
            if let Some(level) = crate::utils::runique_log::get_log().session {
                crate::runique_log!(
                    level,
                    "Large session record ({} bytes) — avoid storing files or images in session",
                    size
                );
            }
        }

        guard.insert(record.id, record.clone());
        self.size_bytes.fetch_add(size, Ordering::Relaxed);
        Ok(())
    }

    async fn save(&self, record: &Record) -> session_store::Result<()> {
        let mut guard = self.data.lock().await;
        let old_size = guard.get(&record.id).map_or(0, estimate_size);
        let new_size = estimate_size(record);

        // Exclusive login: if user_id appears for the first time on this session,
        // invalidate all other sessions for the same user.
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
                .and_then(serde_json::Value::as_i64)
            {
                if !had_user {
                    let mut freed = 0usize;
                    let to_delete: Vec<Id> = guard
                        .iter()
                        .filter(|(id, r)| {
                            **id != record.id
                                && r.data
                                    .get(crate::utils::constante::session_key::session::SESSION_USER_ID_KEY)
                                    .and_then(serde_json::Value::as_i64)
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
                        if let Some(level) = crate::utils::runique_log::get_log().exclusive_login {
                            crate::runique_log!(
                                level,
                                user_id = user_id,
                                "exclusive_login: {} session(s) invalidated for user {}",
                                freed,
                                user_id
                            );
                        }
                    }
                }
            }
        }

        if new_size > MAX_SESSION_RECORD_SIZE {
            if let Some(level) = crate::utils::runique_log::get_log().session {
                crate::runique_log!(
                    level,
                    "Large session record ({} bytes) — avoid storing files or images in session",
                    new_size
                );
            }
        }

        guard.insert(record.id, record.clone());

        if new_size >= old_size {
            self.size_bytes
                .fetch_add(new_size.saturating_sub(old_size), Ordering::Relaxed);
        } else {
            self.size_bytes
                .fetch_sub(old_size.saturating_sub(new_size), Ordering::Relaxed);
        }
        Ok(())
    }

    async fn load(&self, session_id: &Id) -> session_store::Result<Option<Record>> {
        Ok(self
            .data
            .lock()
            .await
            .get(session_id)
            .filter(|r| r.expiry_date > OffsetDateTime::now_utc())
            .cloned())
    }

    async fn delete(&self, session_id: &Id) -> session_store::Result<()> {
        let mut guard = self.data.lock().await;
        if let Some(r) = guard.remove(session_id) {
            self.size_bytes
                .fetch_sub(estimate_size(&r), Ordering::Relaxed);
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
