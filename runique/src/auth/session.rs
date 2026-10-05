//! User session, admin authentication, and authentication traits.
use crate::auth::permissions::{Groupe, Permission};
use crate::auth::user_trait::RuniqueUser;
use crate::middleware::security::csrf::rotate_csrf_token;
use crate::middleware::session::session_db::RuniqueSessionStore;
use crate::utils::aliases::ADb;
use crate::utils::config::TraceResult;
use crate::utils::constante::session_key::session::{
    SESSION_ACTIVE_KEY, SESSION_USER_ID_KEY, SESSION_USER_IS_STAFF_KEY,
    SESSION_USER_IS_SUPERUSER_KEY, SESSION_USER_USERNAME_KEY,
};
use crate::utils::pk::Pk;
use serde::{Deserialize, Serialize};
use tower_sessions::Session;

/// Default when the builder never set a duration (24h), so `login` never breaks
/// even when called outside a full build (tests, library usage).
const DEFAULT_AUTH_SESSION_TTL_SECS: i64 = 86_400;

/// Lifetime of an authenticated session (cookie AND DB row), in seconds.
/// Set **once** at build time from `MiddlewareStaging.session_duration`
/// (builder `.with_session_duration(...)`). Single source → the cookie, the
/// `eihwaz_sessions` row, and the per-request refresh can no longer diverge.
static AUTH_SESSION_TTL_SECS: std::sync::OnceLock<i64> = std::sync::OnceLock::new();

/// Called once at build time. Idempotent. A second call with a **different**
/// value (two apps in the same process) is **logged**, never swallowed.
pub fn set_auth_session_ttl_secs(secs: i64) {
    match AUTH_SESSION_TTL_SECS.get() {
        None => {
            if AUTH_SESSION_TTL_SECS.set(secs).is_err() {
                tracing::warn!("auth session TTL set raced at build — keeping the first value");
            }
        }
        Some(&existing) if existing != secs => {
            tracing::warn!(
                existing,
                attempted = secs,
                "auth session TTL already set to a different value (multi-app in one process?) — keeping the first"
            );
        }
        Some(_) => {}
    }
}

/// Pure TTL resolution (testable without touching the global).
fn resolve_ttl_secs(configured: Option<i64>) -> i64 {
    configured.unwrap_or(DEFAULT_AUTH_SESSION_TTL_SECS)
}

/// Effective TTL for authenticated sessions (builder value, else default).
fn auth_session_ttl_secs() -> i64 {
    resolve_ttl_secs(AUTH_SESSION_TTL_SECS.get().copied())
}

#[cfg(test)]
mod ttl_tests {
    use super::{DEFAULT_AUTH_SESSION_TTL_SECS, resolve_ttl_secs};

    #[test]
    fn ttl_uses_builder_value_else_default() {
        // Builder set a duration → use it as-is (cookie + DB stay aligned).
        assert_eq!(resolve_ttl_secs(Some(172_800)), 172_800);
        // No builder → explicit 24h default, never 0/a panic.
        assert_eq!(resolve_ttl_secs(None), DEFAULT_AUTH_SESSION_TTL_SECS);
        assert_eq!(DEFAULT_AUTH_SESSION_TTL_SECS, 86_400);
    }
}

// ═══════════════════════════════════════════════════════════════
// CurrentUser
// ═══════════════════════════════════════════════════════════════

/// Authenticated user injected into request extensions.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CurrentUser {
    pub id: Pk,
    pub username: String,
    /// Access to the admin panel (read / limited operations)
    pub is_staff: bool,
    /// Full access — bypasses all admin restrictions
    pub is_superuser: bool,
    /// User groups (each group contains its permissions)
    pub groupes: Vec<Groupe>,
}

impl CurrentUser {
    /// Returns the aggregated permission for a single resource (logical OR across all groups).
    /// Returns `None` if the user has no entry for that resource.
    #[must_use]
    pub fn permission_for(&self, resource_key: &str) -> Option<Permission> {
        self.permissions_effectives()
            .into_iter()
            .find(|p| p.resource_key == resource_key)
    }

    /// Returns the effective CRUD permissions (logical OR across all groups, all resources).
    pub fn permissions_effectives(&self) -> Vec<Permission> {
        let mut agg: std::collections::HashMap<String, Permission> =
            std::collections::HashMap::new();
        for groupe in &self.groupes {
            for perm in &groupe.permissions {
                agg.entry(perm.resource_key.clone())
                    .or_insert_with(|| Permission::zeroed(perm.resource_key.clone()))
                    .merge_from(perm);
            }
        }
        agg.into_values().collect()
    }

    /// Checks if the user has a strict global permission (can be refined).
    #[must_use]
    pub fn can_access_resource(&self, resource_key: &str) -> bool {
        self.is_superuser
            || self
                .permission_for(resource_key)
                .is_some_and(|p| p.can_read)
    }

    /// Checks if the user can access the admin panel.
    #[must_use]
    pub fn can_access_admin(&self) -> bool {
        self.is_staff || self.is_superuser
    }
}

// ═══════════════════════════════════════════════════════════════
// Session helpers
// ═══════════════════════════════════════════════════════════════

async fn session_bool(session: &Session, key: &str) -> bool {
    session
        .get::<bool>(key)
        .await
        .ok()
        .flatten()
        .unwrap_or(false)
}

/// Checks if the user is authenticated.
pub async fn is_authenticated(session: &Session) -> bool {
    session
        .get::<Pk>(SESSION_USER_ID_KEY)
        .await
        .ok()
        .flatten()
        .is_some()
}

/// Checks if the user is authenticated and has admin access.
pub async fn is_admin_authenticated(session: &Session) -> bool {
    is_authenticated(session).await
        && (session_bool(session, SESSION_USER_IS_STAFF_KEY).await
            || session_bool(session, SESSION_USER_IS_SUPERUSER_KEY).await)
}

/// Retrieves the ID of the logged-in user.
pub async fn get_user_id(session: &Session) -> Option<Pk> {
    session.get::<Pk>(SESSION_USER_ID_KEY).await.ok().flatten()
}

/// Retrieves the username of the logged-in user.
pub async fn get_username(session: &Session) -> Option<String> {
    session
        .get::<String>(SESSION_USER_USERNAME_KEY)
        .await
        .ok()
        .flatten()
}

// ═══════════════════════════════════════════════════════════════
// Unified Login
// ═══════════════════════════════════════════════════════════════

/// Logs in a user: stores their identity in the session. Their rights and
/// account state are read from the database on each request, not stored here.
///
/// If `db_store` is provided, persists the session in DB (multi-device).
/// If `exclusive` is `true`, invalidates other sessions for the user.
///
/// ```rust,ignore
/// login(&session, &user, None, false).await?;
/// ```
pub async fn login(
    session: &Session,
    user: &impl RuniqueUser,
    db_store: Option<&RuniqueSessionStore>,
    exclusive: bool,
) -> Result<(), tower_sessions::session::Error> {
    let user_id = user.user_id();
    let username = user.username();
    let is_staff = user.is_staff();
    let is_superuser = user.is_superuser();

    // If another session is already active, perform a clean logout before login
    let existing_id: Option<_> = session.get::<Pk>(SESSION_USER_ID_KEY).await.ok().flatten();
    let is_privilege_elevation = existing_id != Some(user_id);
    if let Some(existing) = existing_id
        && existing != user_id
    {
        logout(session, db_store).await.trace(
            crate::utils::runique_log::get_log()
                .session
                .as_ref()
                .and_then(|s| s.store),
            "pre-login logout of previous session",
        );
    }

    // Rotate the session ID on any privilege elevation (anonymous → auth, or user switch).
    // Prevents session fixation: an attacker who planted a session ID cannot reuse it after login.
    if is_privilege_elevation {
        session.cycle_id().await.trace_or(
            crate::utils::runique_log::get_log()
                .session
                .as_ref()
                .and_then(|s| s.store),
            tracing::Level::WARN,
            "cycle session id (session fixation protection)",
        );
        // The CSRF token has to go with the old session id: whoever planted
        // the anonymous session knows its token too.
        rotate_csrf_token(session).await?;
    }

    if let Some(level) = crate::utils::runique_log::get_log()
        .auth
        .as_ref()
        .and_then(|a| a.login)
    {
        crate::runique_log!(
            level,
            user_id = %user_id,
            username = %username,
            is_superuser,
            exclusive,
            db_persist = db_store.is_some(),
            "login"
        );
    }
    session.insert(SESSION_USER_ID_KEY, user_id).await?;
    session
        .insert(SESSION_USER_USERNAME_KEY, username.to_string())
        .await?;
    session.insert(SESSION_USER_IS_STAFF_KEY, is_staff).await?;
    session
        .insert(SESSION_USER_IS_SUPERUSER_KEY, is_superuser)
        .await?;

    // Promote the session TTL to the authenticated duration on the login request
    // itself, so the first persisted row already carries the long expiry instead of
    // the 5-min anonymous window. The ttl-upgrade middleware only kicks in from the
    // next request; without this, a restart in that first window logs the user out.
    let ttl_secs = auth_session_ttl_secs();
    session.set_expiry(Some(tower_sessions::Expiry::OnInactivity(
        tower_sessions::cookie::time::Duration::seconds(ttl_secs),
    )));

    // DB persistence
    if let Some(store) = db_store {
        // cycle_id() (session fixation protection) invalide l'ID courant et diffère la
        // régénération du nouvel ID au prochain save(). Sans ce save() explicite,
        // session.id() == None ici → cookie_id vide → collision sur la contrainte unique
        // eihwaz_sessions_cookie_id_key dès le 2ᵉ login.
        session.save().await?;

        let Some(cookie_id) = session.id().map(|id| id.to_string()) else {
            crate::runique_log!(
                tracing::Level::WARN,
                user_id = %user_id,
                "session id unavailable after save — skipping DB persistence"
            );
            return Ok(());
        };
        let session_id = uuid::Uuid::new_v4().to_string();
        let expires_at = chrono::Utc::now()
            .naive_utc()
            .checked_add_signed(chrono::Duration::seconds(ttl_secs))
            .unwrap_or_else(|| chrono::Utc::now().naive_utc());

        store
            .create(&cookie_id, user_id, &session_id, expires_at)
            .await
            .trace_or(
                crate::utils::runique_log::get_log()
                    .session
                    .as_ref()
                    .and_then(|s| s.store),
                tracing::Level::WARN,
                "persist session to DB",
            );

        if exclusive {
            store
                .invalidate_other_sessions(user_id, &cookie_id)
                .await
                .trace_or(
                    crate::utils::runique_log::get_log()
                        .session
                        .as_ref()
                        .and_then(|s| s.exclusive_login),
                    tracing::Level::WARN,
                    "invalidate other sessions (exclusive login)",
                );
        }
    }

    Ok(())
}

/// Logs in a user starting only from their `user_id` — loads data from the DB.
///
/// Generic shortcut for any authentication flow (registration, OAuth, magic link...)
/// that already has the user identifier without needing to re-send the fields.
///
/// Returns `Ok(())` without creating a session if the account is inactive (`is_active = false`).
///
/// Looks the account up in `eihwaz_users` ([`BuiltinUserEntity`]).
pub async fn auth_login(
    session: &Session,
    db: &ADb,
    user_id: Pk,
) -> Result<(), tower_sessions::session::Error> {
    let Some(user) = crate::auth::user::BuiltinUserEntity::find_by_id(db, user_id).await else {
        return Ok(());
    };
    if !user.can_sign_in() {
        return Ok(());
    }
    let store = RuniqueSessionStore::new(db.clone());
    login(session, &user, Some(&store), false).await
}

/// Logs out a user — removes the memory session and the DB entry if provided.
/// Does nothing when no one is logged in.
pub async fn logout(
    session: &Session,
    db_store: Option<&RuniqueSessionStore>,
) -> Result<(), tower_sessions::session::Error> {
    // The password reset page calls this on every visit, usually from an
    // anonymous session: flushing it would only throw away its CSRF token, and
    // a form shown again on the same page would then carry a dead one.
    if session
        .get::<Pk>(SESSION_USER_ID_KEY)
        .await
        .ok()
        .flatten()
        .is_none()
    {
        return Ok(());
    }

    // DB deletion before clearing the session (cookie_id still accessible)
    if let Some(store) = db_store
        && let Some(cookie_id) = session.id().map(|id| id.to_string())
    {
        store.delete(&cookie_id).await.trace(
            crate::utils::runique_log::get_log()
                .session
                .as_ref()
                .and_then(|s| s.store),
            "delete session from DB on logout",
        );
    }

    // `delete()` alone only drops the stored copy: the request's own copy still
    // holds its id and data, and the session layer saves it right back at the
    // end of the request. `flush()` empties that copy too, the CSRF token with
    // it. It keeps the record's old id though, and the store only picks a new
    // one on a collision; since the CSRF middleware writes a new token before
    // the response leaves, the session would be recreated under the same id.
    // `cycle_id()` gives it a fresh one.
    session.flush().await?;
    session.cycle_id().await
}

/// Protects an anonymous session from cleanup.
pub async fn protect_session(
    session: &Session,
    duration_secs: i64,
) -> Result<(), tower_sessions::session::Error> {
    let protect_until = chrono::Utc::now().timestamp().saturating_add(duration_secs);
    session.insert(SESSION_ACTIVE_KEY, protect_until).await
}

/// Removes manual protection from an anonymous session.
pub async fn unprotect_session(session: &Session) -> Result<(), tower_sessions::session::Error> {
    session.remove::<i64>(SESSION_ACTIVE_KEY).await?;
    Ok(())
}

// ═══════════════════════════════════════════════════════════════
// Axum Middlewares
// ═══════════════════════════════════════════════════════════════
