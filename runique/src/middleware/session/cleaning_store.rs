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
// Constantes
// ═══════════════════════════════════════════════════════════════

/// Déclenche un cleanup proactif des sessions anonymes expirées.
const LOW_WATERMARK_DEFAULT: usize = 128 * 1024 * 1024; // 128 Mo

/// Déclenche un cleanup d'urgence synchrone + refuse les nouvelles sessions si insuffisant.
const HIGH_WATERMARK_DEFAULT: usize = 256 * 1024 * 1024; // 256 Mo

/// Alerte dans les logs si un record de session dépasse cette taille.
const MAX_SESSION_RECORD_SIZE: usize = 50 * 1024; // 50 Ko

// ═══════════════════════════════════════════════════════════════
// Helpers internes
// ═══════════════════════════════════════════════════════════════

/// Estimation de la taille en mémoire d'un record.
/// UUID (16o) + `expiry_date` (8o) + données JSON sérialisées.
fn estimate_size(record: &Record) -> usize {
    24 + serde_json::to_string(&record.data)
        .map(|s| s.len())
        .unwrap_or(256)
}

/// Une session est protégée si elle appartient à un utilisateur authentifié
/// (`user_id` présent) ou si le dev a posé `session_active` avec un timestamp futur.
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

/// `MemoryStore` avec purge automatique et protection par watermarks.
///
/// # Comportement
///
/// **Timer (60s)** : purge toutes les sessions expirées (anonymes et authentifiées).
///
/// **Low watermark** : purge asynchrone (non-bloquante) des sessions anonymes expirées.
///
/// **High watermark** : purge synchrone d'urgence.
///   - Passe 1 : sessions anonymes expirées
///   - Passe 2 : toutes les sessions expirées (si toujours dépassé)
///   - Si encore dépassé → `Error::Backend` (503 pour le client)
///
/// # Protection des sessions
///
/// Sessions avec `user_id` ou `session_active` (timestamp futur) ne sont jamais
/// supprimées sous pression — seules les sessions anonymes sans valeur sont sacrifiées.
#[derive(Clone, Debug)]
pub struct CleaningMemoryStore {
    data: Arc<Mutex<HashMap<Id, Record>>>,
    size_bytes: Arc<AtomicUsize>,
    low_watermark: usize,
    high_watermark: usize,
    /// Si `true`, toute nouvelle connexion invalide les sessions existantes du même utilisateur.
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
    /// Configure les watermarks mémoire.
    ///
    /// - `low`  : déclenche un cleanup proactif (non-bloquant)
    /// - `high` : déclenche un cleanup d'urgence + refuse si insuffisant
    #[must_use]
    pub fn with_watermarks(mut self, low: usize, high: usize) -> Self {
        self.low_watermark = low;
        self.high_watermark = high;
        self
    }

    /// Active ou désactive la connexion exclusive.
    ///
    /// Si `true`, toute nouvelle connexion invalide automatiquement les sessions
    /// existantes du même utilisateur — un seul appareil connecté à la fois.
    #[must_use]
    pub fn with_exclusive_login(mut self, exclusive: bool) -> Self {
        self.exclusive_login = exclusive;
        self
    }

    /// Taille estimée actuelle du store en octets.
    #[must_use]
    pub fn size_bytes(&self) -> usize {
        self.size_bytes.load(Ordering::Relaxed)
    }

    /// Invalide toutes les sessions actives d'un utilisateur.
    ///
    /// Utilisé pour implémenter la connexion exclusive (un seul appareil à la fois).
    /// La session courante (nouvelle connexion) ne doit pas encore contenir `user_id`
    /// au moment de l'appel — elle est donc préservée.
    pub async fn invalidate_user_sessions(&self, user_id: crate::utils::pk::UserId) {
        let mut guard = self.data.lock().await;
        let mut freed = 0usize;
        #[allow(clippy::useless_conversion)]
        let uid_i64: i64 = user_id.into();
        let to_delete: Vec<Id> = guard
            .iter()
            .filter(|(_, r)| {
                r.data
                    .get(crate::utils::constante::SESSION_USER_ID_KEY)
                    .and_then(serde_json::Value::as_i64)
                    .is_some_and(|id| id == uid_i64)
            })
            .map(|(id, _)| *id)
            .collect();
        for id in to_delete {
            if let Some(r) = guard.remove(&id) {
                freed += estimate_size(&r);
            }
        }
        if freed > 0 {
            self.size_bytes.fetch_sub(freed, Ordering::Relaxed);
        }
    }

    /// Spawne la tâche Tokio de cleanup périodique.
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

    /// Purge asynchrone des sessions anonymes expirées (réponse au low watermark).
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
                freed += estimate_size(&r);
            }
        }

        if freed > 0 {
            self.size_bytes.fetch_sub(freed, Ordering::Relaxed);
            if let Some(level) = crate::utils::runique_log::get_log().session {
                crate::runique_log!(
                    level,
                    "Low watermark: {} octets libérés (sessions anonymes expirées)",
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

        // Low watermark — cleanup non-bloquant en arrière-plan
        if current >= self.low_watermark && current < self.high_watermark {
            let store = self.clone();
            tokio::spawn(async move {
                store.purge_anonymous_expired().await;
            });
        }

        let mut guard = self.data.lock().await;

        // High watermark — cleanup d'urgence synchrone sous le lock
        if self.size_bytes.load(Ordering::Relaxed) >= self.high_watermark {
            let now = OffsetDateTime::now_utc();

            // Passe 1 : sessions anonymes expirées
            let mut freed = 0usize;
            let to_delete: Vec<Id> = guard
                .iter()
                .filter(|(_, r)| !is_protected(r) && r.expiry_date <= now)
                .map(|(id, _)| *id)
                .collect();
            for id in to_delete {
                if let Some(r) = guard.remove(&id) {
                    freed += estimate_size(&r);
                }
            }
            self.size_bytes.fetch_sub(freed, Ordering::Relaxed);

            // Passe 2 : toutes les sessions expirées (si toujours au-dessus)
            if self.size_bytes.load(Ordering::Relaxed) >= self.high_watermark {
                let mut freed2 = 0usize;
                let to_delete2: Vec<Id> = guard
                    .iter()
                    .filter(|(_, r)| r.expiry_date <= now)
                    .map(|(id, _)| *id)
                    .collect();
                for id in to_delete2 {
                    if let Some(r) = guard.remove(&id) {
                        freed2 += estimate_size(&r);
                    }
                }
                self.size_bytes.fetch_sub(freed2, Ordering::Relaxed);
                if let Some(level) = crate::utils::runique_log::get_log().session {
                    crate::runique_log!(
                        level,
                        "High watermark: {} + {} octets libérés en urgence",
                        freed,
                        freed2
                    );
                }
            }

            // Toujours au-dessus → refus
            if self.size_bytes.load(Ordering::Relaxed) >= self.high_watermark {
                if let Some(level) = crate::utils::runique_log::get_log().session {
                    crate::runique_log!(
                        level,
                        "Session store saturé ({} octets), nouvelle session refusée",
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
                    "Session record volumineux ({} octets) — évitez de stocker des fichiers ou images en session",
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

        // Connexion exclusive : si user_id apparaît pour la première fois sur cette session,
        // invalider toutes les autres sessions du même utilisateur.
        if self.exclusive_login {
            let had_user = guard
                .get(&record.id)
                .and_then(|r| r.data.get(crate::utils::constante::SESSION_USER_ID_KEY))
                .is_some();
            if let Some(user_id) = record
                .data
                .get(crate::utils::constante::SESSION_USER_ID_KEY)
                .and_then(serde_json::Value::as_i64)
            {
                if !had_user {
                    let mut freed = 0usize;
                    let to_delete: Vec<Id> = guard
                        .iter()
                        .filter(|(id, r)| {
                            **id != record.id
                                && r.data
                                    .get(crate::utils::constante::SESSION_USER_ID_KEY)
                                    .and_then(serde_json::Value::as_i64)
                                    .is_some_and(|id| id == user_id)
                        })
                        .map(|(id, _)| *id)
                        .collect();
                    for id in to_delete {
                        if let Some(r) = guard.remove(&id) {
                            freed += estimate_size(&r);
                        }
                    }
                    if freed > 0 {
                        self.size_bytes.fetch_sub(freed, Ordering::Relaxed);
                        if let Some(level) = crate::utils::runique_log::get_log().exclusive_login {
                            crate::runique_log!(
                                level,
                                user_id = user_id,
                                "exclusive_login: {} session(s) invalidée(s) pour l'utilisateur {}",
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
                    "Session record volumineux ({} octets) — évitez de stocker des fichiers ou images en session",
                    new_size
                );
            }
        }

        guard.insert(record.id, record.clone());

        if new_size >= old_size {
            self.size_bytes
                .fetch_add(new_size - old_size, Ordering::Relaxed);
        } else {
            self.size_bytes
                .fetch_sub(old_size - new_size, Ordering::Relaxed);
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
                freed += estimate_size(r);
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
