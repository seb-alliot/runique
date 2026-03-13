use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

/// Entrée par username : (nombre d'échecs, début du verrouillage)
type Store = Arc<Mutex<HashMap<String, (u32, Instant)>>>;

/// Protection contre le brute-force par username.
///
/// Suit les tentatives de login échouées par compte, indépendamment de l'IP.
/// Aucun faux positif lié au NAT ou aux proxies partagés.
///
/// # Exemple
/// ```rust,ignore
/// use runique::prelude::*;
/// use std::sync::Arc;
///
/// let guard = Arc::new(LoginGuard::new(5, 300)); // 5 échecs → 5 min de blocage
///
/// // Dans le handler de login :
/// if guard.is_locked(&username) {
///     // retourner une erreur de blocage
/// }
/// match authenticate(&username, &password, &db).await {
///     Some(user) => {
///         guard.record_success(&username);
///         login(&session, user.id, &user.username).await?;
///     }
///     None => {
///         guard.record_failure(&username);
///     }
/// }
/// ```
#[derive(Clone)]
pub struct LoginGuard {
    store: Store,
    /// Nombre d'échecs avant verrouillage
    pub max_attempts: u32,
    /// Durée du verrouillage en secondes
    pub lockout_secs: u64,
}

impl LoginGuard {
    pub fn new(max_attempts: u32, lockout_secs: u64) -> Self {
        Self {
            store: Arc::new(Mutex::new(HashMap::new())),
            max_attempts,
            lockout_secs,
        }
    }

    /// Construit depuis les variables d'environnement :
    /// `RUNIQUE_LOGIN_MAX_ATTEMPTS` (défaut : 5)
    /// `RUNIQUE_LOGIN_LOCKOUT_SECS` (défaut : 300)
    pub fn from_env() -> Self {
        let max = std::env::var("RUNIQUE_LOGIN_MAX_ATTEMPTS")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(5);
        let lockout = std::env::var("RUNIQUE_LOGIN_LOCKOUT_SECS")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(300);
        Self::new(max, lockout)
    }

    /// Enregistre un échec de connexion pour ce username
    pub fn record_failure(&self, username: &str) {
        let mut store = match self.store.lock() {
            Ok(s) => s,
            Err(p) => p.into_inner(),
        };
        let entry = store
            .entry(username.to_string())
            .or_insert((0, Instant::now()));
        entry.0 += 1;
        entry.1 = Instant::now();
    }

    /// Réinitialise le compteur après une connexion réussie
    pub fn record_success(&self, username: &str) {
        let mut store = match self.store.lock() {
            Ok(s) => s,
            Err(p) => p.into_inner(),
        };
        store.remove(username);
    }

    /// Retourne `true` si le compte est temporairement verrouillé
    pub fn is_locked(&self, username: &str) -> bool {
        let store = match self.store.lock() {
            Ok(s) => s,
            Err(p) => p.into_inner(),
        };
        if let Some((attempts, last)) = store.get(username) {
            if *attempts >= self.max_attempts {
                return last.elapsed() < Duration::from_secs(self.lockout_secs);
            }
        }
        false
    }

    /// Nombre d'échecs en cours pour ce username
    pub fn attempts(&self, username: &str) -> u32 {
        let store = match self.store.lock() {
            Ok(s) => s,
            Err(p) => p.into_inner(),
        };
        store.get(username).map(|(n, _)| *n).unwrap_or(0)
    }

    /// Secondes restantes avant déverrouillage, ou `None` si non verrouillé
    pub fn remaining_lockout_secs(&self, username: &str) -> Option<u64> {
        let store = match self.store.lock() {
            Ok(s) => s,
            Err(p) => p.into_inner(),
        };
        if let Some((attempts, last)) = store.get(username) {
            if *attempts >= self.max_attempts {
                let elapsed = last.elapsed().as_secs();
                if elapsed < self.lockout_secs {
                    return Some(self.lockout_secs - elapsed);
                }
            }
        }
        None
    }
}
