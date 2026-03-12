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
/// UUID (16o) + expiry_date (8o) + données JSON sérialisées.
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
            .and_then(|v| v.as_i64())
            .map(|ts| ts > OffsetDateTime::now_utc().unix_timestamp())
            .unwrap_or(false)
}

// ═══════════════════════════════════════════════════════════════
// CleaningMemoryStore
// ═══════════════════════════════════════════════════════════════

/// MemoryStore avec purge automatique et protection par watermarks.
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
}

impl Default for CleaningMemoryStore {
    fn default() -> Self {
        Self {
            data: Arc::new(Mutex::new(HashMap::new())),
            size_bytes: Arc::new(AtomicUsize::new(0)),
            low_watermark: LOW_WATERMARK_DEFAULT,
            high_watermark: HIGH_WATERMARK_DEFAULT,
        }
    }
}

impl CleaningMemoryStore {
    /// Configure les watermarks mémoire.
    ///
    /// - `low`  : déclenche un cleanup proactif (non-bloquant)
    /// - `high` : déclenche un cleanup d'urgence + refuse si insuffisant
    pub fn with_watermarks(mut self, low: usize, high: usize) -> Self {
        self.low_watermark = low;
        self.high_watermark = high;
        self
    }

    /// Taille estimée actuelle du store en octets.
    pub fn size_bytes(&self) -> usize {
        self.size_bytes.load(Ordering::Relaxed)
    }

    /// Spawne la tâche Tokio de cleanup périodique.
    pub fn spawn_cleanup(&self, period: tokio::time::Duration) {
        let store = self.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(period);
            loop {
                interval.tick().await;
                if let Err(e) = store.delete_expired().await {
                    tracing::error!("session cleanup error: {e}");
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
            tracing::warn!(
                "Low watermark: {} octets libérés (sessions anonymes expirées)",
                freed
            );
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
                tracing::error!(
                    "High watermark: {} + {} octets libérés en urgence",
                    freed,
                    freed2
                );
            }

            // Toujours au-dessus → refus
            if self.size_bytes.load(Ordering::Relaxed) >= self.high_watermark {
                tracing::error!(
                    "Session store saturé ({} octets), nouvelle session refusée",
                    self.size_bytes()
                );
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
            tracing::warn!(
                "Session record volumineux ({} octets) — évitez de stocker des fichiers ou images en session",
                size
            );
        }

        guard.insert(record.id, record.clone());
        self.size_bytes.fetch_add(size, Ordering::Relaxed);
        Ok(())
    }

    async fn save(&self, record: &Record) -> session_store::Result<()> {
        let mut guard = self.data.lock().await;
        let old_size = guard.get(&record.id).map(estimate_size).unwrap_or(0);
        let new_size = estimate_size(record);

        if new_size > MAX_SESSION_RECORD_SIZE {
            tracing::warn!(
                "Session record volumineux ({} octets) — évitez de stocker des fichiers ou images en session",
                new_size
            );
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
