//! Brute-force protection, `login_required` middleware, and permission cache.
use crate::admin::permissions::Groupe;
use crate::utils::pk::Pk;
use std::{
    collections::HashMap,
    sync::{Arc, LazyLock, Mutex, RwLock},
    time::{Duration, Instant},
};
use tokio::time::interval;

// ═══════════════════════════════════════════════════════════════
// Global permission cache by user_id
// ═══════════════════════════════════════════════════════════════

#[derive(Clone, Debug)]
pub struct CachedPermissions {
    pub groupes: Vec<Groupe>,
}

static PERMISSIONS_CACHE: LazyLock<RwLock<HashMap<Pk, Arc<CachedPermissions>>>> =
    LazyLock::new(|| RwLock::new(HashMap::new()));

/// Inserts or updates a user's permissions in the cache.
/// Called upon login and when a rights change signal occurs.
pub fn cache_permissions(user_id: Pk, groupes: Vec<Groupe>) {
    if let Ok(mut cache) = PERMISSIONS_CACHE.write() {
        cache.insert(user_id, Arc::new(CachedPermissions { groupes }));
    }
}

/// Returns the cached permissions for a user.
pub fn get_permissions(user_id: Pk) -> Option<Arc<CachedPermissions>> {
    PERMISSIONS_CACHE.read().ok()?.get(&user_id).cloned()
}

/// Removes a user's permissions from the cache (logout).
pub fn evict_permissions(user_id: Pk) {
    if let Ok(mut cache) = PERMISSIONS_CACHE.write() {
        cache.remove(&user_id);
    }
}

/// Entirely clears the cache (restart, maintenance).
pub fn clear_cache() {
    if let Ok(mut cache) = PERMISSIONS_CACHE.write() {
        cache.clear();
    }
}

// ═══════════════════════════════════════════════════════════════
// LoginGuard
// ═══════════════════════════════════════════════════════════════

/// Entry by username: (number of failures, start of lockout)
type Store = Arc<Mutex<HashMap<String, (u32, Instant)>>>;

/// Brute-force protection by username.
///
/// Tracks failed login attempts per account, independent of IP.
/// No false positives related to NAT or shared proxies.
///
/// # Example
/// ```rust,ignore
/// use runique::prelude::*;
/// use std::sync::Arc;
///
/// let guard = Arc::new(LoginGuard::new().max_attempts(5).lockout_secs(300));
///
/// // In the login handler:
/// if guard.is_locked(&username) {
///     // return a blocking error
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
    /// Number of failures before lockout
    pub max_attempts: u32,
    /// Lockout duration in seconds
    pub lockout_secs: u64,
}

impl LoginGuard {
    /// Creates a `LoginGuard` with default values (5 attempts / 300 s).
    pub fn new() -> Self {
        Self {
            store: Arc::new(Mutex::new(HashMap::new())),
            max_attempts: 5,
            lockout_secs: 300,
        }
    }

    /// Number of failures before account lockout
    #[must_use]
    pub fn max_attempts(mut self, max: u32) -> Self {
        self.max_attempts = max;
        self
    }

    /// Lockout duration in seconds
    #[must_use]
    pub fn lockout_secs(mut self, secs: u64) -> Self {
        self.lockout_secs = secs;
        self
    }

    /// Spawns a Tokio task that periodically purges expired entries.
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

    /// Records a connection failure for this username
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

    /// Resets the counter after a successful connection
    pub fn record_success(&self, username: &str) {
        let mut store = match self.store.lock() {
            Ok(s) => s,
            Err(p) => p.into_inner(),
        };
        store.remove(username);
    }

    /// Returns `true` if the account is temporarily locked
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

    /// Current number of failures for this username
    #[must_use]
    pub fn attempts(&self, username: &str) -> u32 {
        let store = match self.store.lock() {
            Ok(s) => s,
            Err(p) => p.into_inner(),
        };
        store.get(username).map_or(0, |(n, _)| *n)
    }

    /// Returns the effective key to use with `LoginGuard`.
    ///
    /// - Non-empty username → key by username (targeted account protection)
    /// - Empty or absent username → `"anonym:{ip}"` (anonymous protection by IP)
    #[must_use]
    pub fn effective_key<'a>(username: &'a str, ip: &str) -> std::borrow::Cow<'a, str> {
        if username.trim().is_empty() {
            std::borrow::Cow::Owned(format!("anonym:{ip}"))
        } else {
            std::borrow::Cow::Borrowed(username)
        }
    }

    /// Remaining seconds before unlocking, or `None` if not locked
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

/// Middleware that redirects to `redirect_url` if the user is not authenticated.
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
