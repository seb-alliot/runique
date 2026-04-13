//! Protection brute-force, middleware `login_required`, et cache de permissions.
use crate::admin::permissions::Groupe;
use crate::utils::pk::Pk;
use std::{
    collections::HashMap,
    sync::{Arc, LazyLock, Mutex, RwLock},
    time::{Duration, Instant},
};
use tokio::time::interval;

// ═══════════════════════════════════════════════════════════════
// Cache global des permissions par user_id
// ═══════════════════════════════════════════════════════════════

#[derive(Clone, Debug)]
pub struct CachedPermissions {
    pub groupes: Vec<Groupe>,
}

static PERMISSIONS_CACHE: LazyLock<RwLock<HashMap<Pk, Arc<CachedPermissions>>>> =
    LazyLock::new(|| RwLock::new(HashMap::new()));

/// Insère ou met à jour les permissions d'un utilisateur dans le cache.
/// Appelé au login et lors d'un signal de changement de droits.
pub fn cache_permissions(user_id: Pk, groupes: Vec<Groupe>) {
    if let Ok(mut cache) = PERMISSIONS_CACHE.write() {
        cache.insert(user_id, Arc::new(CachedPermissions { groupes }));
    }
}

/// Retourne les permissions cachées pour un utilisateur.
pub fn get_permissions(user_id: Pk) -> Option<Arc<CachedPermissions>> {
    PERMISSIONS_CACHE.read().ok()?.get(&user_id).cloned()
}

/// Supprime les permissions d'un utilisateur du cache (logout).
pub fn evict_permissions(user_id: Pk) {
    if let Ok(mut cache) = PERMISSIONS_CACHE.write() {
        cache.remove(&user_id);
    }
}

/// Vide entièrement le cache (redémarrage, maintenance).
pub fn clear_cache() {
    if let Ok(mut cache) = PERMISSIONS_CACHE.write() {
        cache.clear();
    }
}

// ═══════════════════════════════════════════════════════════════
// LoginGuard
// ═══════════════════════════════════════════════════════════════

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
    /// Crée un `LoginGuard` avec les valeurs par défaut (5 tentatives / 300 s).
    pub fn new() -> Self {
        Self {
            store: Arc::new(Mutex::new(HashMap::new())),
            max_attempts: 5,
            lockout_secs: 300,
        }
    }

    /// Nombre d'échecs avant verrouillage du compte
    #[must_use]
    pub fn max_attempts(mut self, max: u32) -> Self {
        self.max_attempts = max;
        self
    }

    /// Durée du verrouillage en secondes
    #[must_use]
    pub fn lockout_secs(mut self, secs: u64) -> Self {
        self.lockout_secs = secs;
        self
    }

    /// Spawne une tâche Tokio qui purge périodiquement les entrées expirées.
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
        entry.0 = entry.0.saturating_add(1);
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
    #[must_use]
    pub fn is_locked(&self, username: &str) -> bool {
        let store = match self.store.lock() {
            Ok(s) => s,
            Err(p) => p.into_inner(),
        };
        let (attempts, last) = store
            .get(username)
            .map_or((0, Instant::now()), |(a, t)| (*a, *t));
        attempts >= self.max_attempts && last.elapsed() < Duration::from_secs(self.lockout_secs)
    }

    /// Nombre d'échecs en cours pour ce username
    #[must_use]
    pub fn attempts(&self, username: &str) -> u32 {
        let store = match self.store.lock() {
            Ok(s) => s,
            Err(p) => p.into_inner(),
        };
        store.get(username).map_or(0, |(n, _)| *n)
    }

    /// Retourne la clé effective à utiliser avec `LoginGuard`.
    ///
    /// - Username non vide → clé par username (protection compte ciblé)
    /// - Username vide ou absent → `"anonym:{ip}"` (protection anonyme par IP)
    #[must_use]
    pub fn effective_key<'a>(username: &'a str, ip: &str) -> std::borrow::Cow<'a, str> {
        if username.trim().is_empty() {
            std::borrow::Cow::Owned(format!("anonym:{ip}"))
        } else {
            std::borrow::Cow::Borrowed(username)
        }
    }

    /// Secondes restantes avant déverrouillage, ou `None` si non verrouillé
    #[must_use]
    pub fn remaining_lockout_secs(&self, username: &str) -> Option<u64> {
        let store = match self.store.lock() {
            Ok(s) => s,
            Err(p) => p.into_inner(),
        };
        if let Some((attempts, last)) = store.get(username) {
            if *attempts >= self.max_attempts {
                let elapsed = last.elapsed().as_secs();
                if elapsed < self.lockout_secs {
                    return Some(self.lockout_secs.saturating_sub(elapsed));
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

// ═══════════════════════════════════════════════════════════════
// login_required middleware
// ═══════════════════════════════════════════════════════════════

use axum::{
    body::Body,
    extract::State,
    http::Request,
    middleware::Next,
    response::{IntoResponse, Redirect, Response},
};
use tower_sessions::Session;

/// Middleware qui redirige vers `redirect_url` si l'utilisateur n'est pas authentifié.
pub(crate) async fn login_required_middleware(
    State(redirect_url): State<Arc<String>>,
    session: Session,
    req: Request<Body>,
    next: Next,
) -> Response {
    if crate::auth::session::is_authenticated(&session).await {
        next.run(req).await
    } else {
        Redirect::to(redirect_url.as_str()).into_response()
    }
}
