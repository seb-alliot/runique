//! Brute-force protection and the `login_required` middleware.
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};
use tokio::time::interval;

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
///         login(&session, &user, None, false).await?;
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
    ///
    /// The admin builder starts this automatically. Standalone uses should
    /// call it once while a Tokio runtime is active.
    pub fn spawn_cleanup(&self, period: tokio::time::Duration) {
        let store = self.store.clone();
        let lockout_secs = self.lockout_secs;
        let period = period.max(tokio::time::Duration::from_secs(1));
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
        if store
            .get(username)
            .is_some_and(|(_, last)| last.elapsed() >= Duration::from_secs(self.lockout_secs))
        {
            store.remove(username);
        }
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
        let mut store = match self.store.lock() {
            Ok(s) => s,
            Err(p) => p.into_inner(),
        };
        let expired = store
            .get(username)
            .is_some_and(|(_, last)| last.elapsed() >= Duration::from_secs(self.lockout_secs));
        if expired {
            store.remove(username);
            return false;
        }
        store
            .get(username)
            .is_some_and(|(attempts, _)| *attempts >= self.max_attempts)
    }

    /// Current number of failures for this username
    #[must_use]
    pub fn attempts(&self, username: &str) -> u32 {
        let mut store = match self.store.lock() {
            Ok(s) => s,
            Err(p) => p.into_inner(),
        };
        let expired = store
            .get(username)
            .is_some_and(|(_, last)| last.elapsed() >= Duration::from_secs(self.lockout_secs));
        if expired {
            store.remove(username);
            0
        } else {
            store.get(username).map_or(0, |(n, _)| *n)
        }
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
        let mut store = match self.store.lock() {
            Ok(s) => s,
            Err(p) => p.into_inner(),
        };
        if store
            .get(username)
            .is_some_and(|(_, last)| last.elapsed() >= Duration::from_secs(self.lockout_secs))
        {
            store.remove(username);
            return None;
        }
        if let Some((attempts, last)) = store.get(username)
            && *attempts >= self.max_attempts
        {
            let elapsed = last.elapsed().as_secs();
            if elapsed < self.lockout_secs {
                return Some(self.lockout_secs.saturating_sub(elapsed));
            }
        }
        None
    }

    /// Cleanup cadence, capped so expired entries aren't retained for a full
    /// long lockout period.
    pub(crate) fn cleanup_interval(&self) -> tokio::time::Duration {
        tokio::time::Duration::from_secs(self.lockout_secs.clamp(1, 60))
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

#[cfg(test)]
mod expiration_tests {
    use super::*;

    #[test]
    fn a_failure_after_expiration_starts_a_fresh_counter() {
        let guard = LoginGuard::new().max_attempts(2).lockout_secs(60);
        {
            let mut store = guard.store.lock().unwrap();
            store.insert(
                "alice".to_string(),
                (2, Instant::now() - Duration::from_secs(61)),
            );
        }

        guard.record_failure("alice");
        assert_eq!(guard.attempts("alice"), 1);
        assert!(!guard.is_locked("alice"));
    }

    #[test]
    fn the_cleanup_cadence_follows_the_lockout_within_bounds() {
        let secs = |lockout| {
            LoginGuard::new()
                .lockout_secs(lockout)
                .cleanup_interval()
                .as_secs()
        };
        assert_eq!(secs(0), 1);
        assert_eq!(secs(30), 30);
        assert_eq!(secs(600), 60);
    }

    // Read straight from the store: `attempts()` would purge the entry itself.
    #[tokio::test]
    async fn the_background_task_removes_expired_entries() {
        let guard = LoginGuard::new().max_attempts(3).lockout_secs(60);
        {
            let mut store = guard.store.lock().unwrap();
            store.insert(
                "expired".to_string(),
                (1, Instant::now() - Duration::from_secs(61)),
            );
            store.insert("fresh".to_string(), (1, Instant::now()));
        }

        guard.spawn_cleanup(tokio::time::Duration::from_secs(1));
        tokio::time::sleep(tokio::time::Duration::from_millis(1200)).await;

        let store = guard.store.lock().unwrap();
        assert!(!store.contains_key("expired"));
        assert!(store.contains_key("fresh"));
    }
}
