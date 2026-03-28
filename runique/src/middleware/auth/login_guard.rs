use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};
use tokio::time::interval;

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
/// let guard = Arc::new(LoginGuard::new().max_attempts(5).lockout_secs(300));
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
    /// Crée un LoginGuard avec les valeurs par défaut (5 tentatives / 300 s).
    ///
    /// # Exemple
    /// ```rust,ignore
    /// LoginGuard::new()
    ///     .max_attempts(5)
    ///     .lockout_secs(300)
    /// ```
    pub fn new() -> Self {
        Self {
            store: Arc::new(Mutex::new(HashMap::new())),
            max_attempts: 5,
            lockout_secs: 300,
        }
    }

    /// Nombre d'échecs avant verrouillage du compte
    pub fn max_attempts(mut self, max: u32) -> Self {
        self.max_attempts = max;
        self
    }

    /// Durée du verrouillage en secondes
    pub fn lockout_secs(mut self, secs: u64) -> Self {
        self.lockout_secs = secs;
        self
    }

    /// Spawne une tâche Tokio qui purge périodiquement les entrées expirées.
    /// À appeler une fois au démarrage de l'application.
    pub fn spawn_cleanup(&self, period: tokio::time::Duration) {
        let store = self.store.clone();
        let lockout_secs = self.lockout_secs;
        tokio::spawn(async move {
            let mut ticker = interval(period);
            loop {
                ticker.tick().await;
                let mut guard = match store.lock() {
                    Ok(g) => g,
                    Err(p) => p.into_inner(),
                };
                let lockout = Duration::from_secs(lockout_secs);
                guard.retain(|_, (_, last)| last.elapsed() < lockout);
            }
        });
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
        // Lecture unifiée : même chemin de code qu'un username connu ou inconnu
        // → réduit la fuite de timing permettant l'énumération d'usernames.
        let (attempts, last) = store
            .get(username)
            .map(|(a, t)| (*a, *t))
            .unwrap_or((0, Instant::now()));
        attempts >= self.max_attempts && last.elapsed() < Duration::from_secs(self.lockout_secs)
    }

    /// Nombre d'échecs en cours pour ce username
    pub fn attempts(&self, username: &str) -> u32 {
        let store = match self.store.lock() {
            Ok(s) => s,
            Err(p) => p.into_inner(),
        };
        store.get(username).map(|(n, _)| *n).unwrap_or(0)
    }

    /// Retourne la clé effective à utiliser avec `LoginGuard`.
    ///
    /// - Username non vide → clé par username (protection compte ciblé)
    /// - Username vide ou absent → `"anonym:{ip}"` (protection anonyme par IP)
    ///
    /// Garantit que les tentatives anonymes ne partagent pas un compteur global :
    /// verrouiller `"anonym:1.2.3.4"` n'affecte pas `"anonym:5.6.7.8"`.
    ///
    /// # Exemple
    /// ```rust,ignore
    /// let key = LoginGuard::effective_key(&username, &ip);
    /// if GUARD.is_locked(&key) { /* 429 */ }
    /// ```
    pub fn effective_key<'a>(username: &'a str, ip: &str) -> std::borrow::Cow<'a, str> {
        if username.trim().is_empty() {
            std::borrow::Cow::Owned(format!("anonym:{ip}"))
        } else {
            std::borrow::Cow::Borrowed(username)
        }
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

impl Default for LoginGuard {
    fn default() -> Self {
        Self::new()
    }
}
