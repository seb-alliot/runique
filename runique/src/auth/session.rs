//! The session mechanics: login, logout, the authenticated session's lifetime and protection.
use crate::auth::user_trait::RuniqueUser;
use crate::middleware::security::csrf::rotate_csrf_token;
use crate::middleware::session::session_db::RuniqueSessionStore;
use crate::utils::config::TraceResult;
use crate::utils::constante::session_key::session::{
    SESSION_ACTIVE_KEY, SESSION_USER_ID_KEY, SESSION_VERSION_KEY,
};
use crate::utils::pk::Pk;
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

#[cfg(test)]
mod session_version_tests {
    use super::{SESSION_VERSION_KEY, session_is_current};
    use crate::auth::user_trait::RuniqueUser;
    use crate::utils::pk::Pk;
    use std::sync::Arc;
    use tower_sessions::{MemoryStore, Session};

    struct Account(Option<i32>);

    impl RuniqueUser for Account {
        fn user_id(&self) -> Pk {
            Pk::default()
        }
        fn username(&self) -> &str {
            "account"
        }
        fn email(&self) -> &str {
            "account@example.com"
        }
        fn is_active(&self) -> bool {
            true
        }
        fn is_superuser(&self) -> bool {
            false
        }
        fn session_version(&self) -> i32 {
            self.0.unwrap_or_default()
        }
    }

    /// A session opened at `version`, or holding none.
    async fn session(version: Option<i32>) -> Session {
        let session = Session::new(None, Arc::new(MemoryStore::default()), None);
        if let Some(version) = version {
            session.insert(SESSION_VERSION_KEY, version).await.unwrap();
        }
        session
    }

    #[tokio::test]
    async fn a_session_is_current_only_at_the_accounts_version() {
        let opened = session(Some(3)).await;
        assert!(session_is_current(&opened, &Account(Some(3))).await);
        assert!(
            !session_is_current(&opened, &Account(Some(4))).await,
            "older"
        );
        assert!(
            !session_is_current(&opened, &Account(Some(2))).await,
            "newer"
        );
    }

    /// A session opened before the column existed lives until the first increase.
    #[tokio::test]
    async fn a_session_without_a_version_ends_with_the_first_increase() {
        let opened = session(None).await;
        assert!(session_is_current(&opened, &Account(Some(0))).await);
        assert!(!session_is_current(&opened, &Account(Some(1))).await);
    }

    #[tokio::test]
    async fn a_user_type_that_declares_no_version_closes_nothing() {
        struct Minimal;
        impl RuniqueUser for Minimal {
            fn user_id(&self) -> Pk {
                Pk::default()
            }
            fn username(&self) -> &str {
                "minimal"
            }
            fn email(&self) -> &str {
                "minimal@example.com"
            }
            fn is_active(&self) -> bool {
                true
            }
            fn is_superuser(&self) -> bool {
                false
            }
        }
        assert_eq!(Minimal.session_version(), 0);
        assert!(session_is_current(&session(None).await, &Minimal).await);
    }
}

// ═══════════════════════════════════════════════════════════════
// Session helpers
// ═══════════════════════════════════════════════════════════════

/// Checks if the user is authenticated.
pub async fn is_authenticated(session: &Session) -> bool {
    session
        .get::<Pk>(SESSION_USER_ID_KEY)
        .await
        .trace(
            crate::utils::runique_log::log_level!(session.store),
            "log.session.read_account",
        )
        .flatten()
        .is_some()
}

/// The id the session was signed in with — for the middlewares that reload
/// the account. Handlers read `request.user`, the account as the database has it.
pub(crate) async fn get_user_id(session: &Session) -> Option<Pk> {
    session
        .get::<Pk>(SESSION_USER_ID_KEY)
        .await
        .trace(
            crate::utils::runique_log::log_level!(session.store),
            "log.session.read_account",
        )
        .flatten()
}

// ═══════════════════════════════════════════════════════════════
// Unified Login
// ═══════════════════════════════════════════════════════════════

/// Logs in a user: stores their id in the session (and renews the session id
/// and the CSRF token). Their name, rights and account state are read from the
/// database on each request (`request.user`), not stored here.
///
/// Refuses an account that may not sign in (`can_sign_in()`: inactive, or never
/// activated) with [`LoginError::CannotSignIn`], writing nothing. The account
/// must come from the server (`authenticate_user`, an account just created or
/// activated), never from an id taken from the request.
///
/// If `db_store` is provided, persists the session in DB (multi-device).
/// `exclusive` says what happens to the account's other sessions
/// ([`ExclusiveLogin`]); `true` and `false` still work, as `On` and `Off`.
///
/// ```rust,ignore
/// login(&session, &user, None, false).await?;
/// ```
pub async fn login(
    session: &Session,
    user: &impl RuniqueUser,
    db_store: Option<&RuniqueSessionStore>,
    exclusive: impl Into<ExclusiveLogin>,
) -> Result<(), LoginError> {
    // Checked here whatever the path: a session is never opened for an inactive
    // or never-activated account, even one loaded without `authenticate_user`.
    if !user.can_sign_in() {
        return Err(LoginError::CannotSignIn);
    }
    let exclusive = exclusive.into();
    let user_id = user.user_id();
    let username = user.username();
    let is_superuser = user.is_superuser();

    // Another account signed in on this browser is logged out first. If that
    // fails its row survives, and with it any stolen copy of its cookie: the
    // new sign-in is refused rather than leave it alive.
    let existing_id: Option<_> = session
        .get::<Pk>(SESSION_USER_ID_KEY)
        .await
        .map_err(LoginError::Session)?;
    let is_privilege_elevation = existing_id != Some(user_id);
    if let Some(existing) = existing_id
        && existing != user_id
    {
        logout(session, db_store)
            .await
            .map_err(LoginError::Session)?;
    }

    // Rotate the session ID on any privilege elevation (anonymous → auth, or user switch).
    // Prevents session fixation: an attacker who planted a session ID cannot reuse it after login.
    // One retry covers a brief store failure. A second failure stops the login
    // before the account id is written onto the old cookie.
    if is_privilege_elevation {
        cycle_session_id(session).await?;
        // The CSRF token has to go with the old session id: whoever planted
        // the anonymous session knows its token too.
        rotate_csrf_token(session).await?;
    }

    if let Some(level) = crate::utils::runique_log::log_level!(auth.login) {
        crate::runique_log!(
            level,
            user_id = %user_id,
            username = %username,
            is_superuser,
            exclusive = ?exclusive,
            db_persist = db_store.is_some(),
            "login"
        );
    }
    // The id alone: the account (name, rights) is read from the database on
    // every request, a copy here would outlive a change.
    session
        .insert(SESSION_VERSION_KEY, user.session_version())
        .await?;
    session.insert(SESSION_USER_ID_KEY, user_id).await?;

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
        // cycle_id() (session fixation protection) invalidates the current ID and defers
        // generating the new one until the next save(). Without this explicit save(),
        // session.id() == None here → empty cookie_id → collision on the unique constraint
        // eihwaz_sessions_cookie_id_key from the second login on.
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
                crate::utils::runique_log::log_level!(session.store),
                tracing::Level::WARN,
                "persist session to DB",
            );

        if exclusive != ExclusiveLogin::Off {
            let closed = store.invalidate_other_sessions(user_id, &cookie_id).await;
            let refused = match &closed {
                Err(error) if exclusive == ExclusiveLogin::Strict => Some(error.to_string()),
                _ => None,
            };
            closed.trace_or(
                crate::utils::runique_log::log_level!(session.exclusive_login),
                tracing::Level::WARN,
                "invalidate other sessions (exclusive login)",
            );
            if let Some(reason) = refused {
                // `Strict` promised a single session: the one just opened is
                // closed too. Not through `logout`, which stops at the first
                // refusal of the store — the very case here. `flush` empties
                // this request's session before it asks the store anything.
                let level = crate::utils::runique_log::log_level!(session.store);
                store.delete(&cookie_id).await.trace_or(
                    level,
                    tracing::Level::WARN,
                    "delete the row of a refused exclusive login",
                );
                session.flush().await.trace_or(
                    level,
                    tracing::Level::WARN,
                    "close the session of a refused exclusive login",
                );
                return Err(LoginError::Session(tower_sessions::session::Error::Store(
                    tower_sessions::session_store::Error::Backend(reason),
                )));
            }
        }
    }

    Ok(())
}

/// Renews the session id. A store failure is logged on `session.store`
/// (`WARN` when that channel is off) and tried once more. The second failure
/// is returned: the caller must not attach an account to the id that stayed.
async fn cycle_session_id(session: &Session) -> Result<(), tower_sessions::session::Error> {
    let level = crate::utils::runique_log::log_level!(session.store);
    if let Err(error) = session.cycle_id().await {
        log_cycle_id_failure(level, &error);
        if let Err(error) = session.cycle_id().await {
            log_cycle_id_failure(level, &error);
            return Err(error);
        }
    }
    Ok(())
}

#[track_caller]
fn log_cycle_id_failure(level: Option<tracing::Level>, error: &impl std::fmt::Debug) {
    let loc = std::panic::Location::caller();
    crate::runique_log!(
        level.unwrap_or(tracing::Level::WARN),
        error = ?error,
        at = %loc,
        "cycle session id (session fixation protection)"
    );
}

/// What a sign-in does to the account's other sessions (other devices).
/// `Strict` is `On`, harder: both close them, only `Strict` refuses the
/// sign-in when closing them failed.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ExclusiveLogin {
    /// The other sessions stay open.
    Off,
    /// The other sessions are closed. If the store refuses, it is logged and
    /// the sign-in goes on.
    On,
    /// The other sessions are closed. If the store refuses, the sign-in is
    /// refused with [`LoginError::Session`] and its own session closed.
    Strict,
}

impl From<bool> for ExclusiveLogin {
    fn from(exclusive: bool) -> Self {
        if exclusive { Self::On } else { Self::Off }
    }
}

/// Why [`login`] opened no session.
#[derive(Debug)]
pub enum LoginError {
    /// The account may not sign in (`can_sign_in()` is false: inactive, or never
    /// activated). Nothing was written to the session.
    CannotSignIn,
    /// The session store refused the write.
    Session(tower_sessions::session::Error),
}

impl std::fmt::Display for LoginError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::CannotSignIn => f.write_str(&crate::utils::trad::t("auth.login_cannot_sign_in")),
            Self::Session(e) => {
                f.write_str(&crate::utils::trad::tf("auth.login_session_failed", &[e]))
            }
        }
    }
}

impl std::error::Error for LoginError {}

impl From<tower_sessions::session::Error> for LoginError {
    fn from(e: tower_sessions::session::Error) -> Self {
        Self::Session(e)
    }
}

/// Logs out a user — removes the memory session and the DB entry if provided.
/// Does nothing when no one is logged in.
/// Whether this session was opened under the account's current version. A
/// session holding none counts as version 0, so it ends with the account's
/// first increase.
pub(crate) async fn session_is_current(session: &Session, user: &impl RuniqueUser) -> bool {
    let opened_at = session
        .get::<i32>(SESSION_VERSION_KEY)
        .await
        .trace(
            crate::utils::runique_log::log_level!(session.store),
            "log.session.read_version",
        )
        .flatten()
        .unwrap_or(0);
    opened_at == user.session_version()
}

pub async fn logout(
    session: &Session,
    db_store: Option<&RuniqueSessionStore>,
) -> Result<(), tower_sessions::session::Error> {
    // The password reset page calls this once its transaction committed,
    // often from an anonymous session: flushing it would only throw away its
    // CSRF token, and a form shown again on the same page would then carry a
    // dead one.
    if session.get::<Pk>(SESSION_USER_ID_KEY).await?.is_none() {
        return Ok(());
    }

    if let Some(store) = db_store
        && let Some(cookie_id) = session.id().map(|id| id.to_string())
    {
        store.delete(&cookie_id).await.map_err(|e| {
            tower_sessions::session::Error::Store(tower_sessions::session_store::Error::Backend(
                e.to_string(),
            ))
        })?;
    }

    // Revoke before clearing: `delete()` leaves this request's data alone, so
    // if the store fails the session stays whole, its cookie too, and the user
    // can try again. `flush()` clears first: a failure there would drop the
    // cookie while the row still lets a stolen copy in.
    session.delete().await?;

    // `flush()` empties this request's copy too, the CSRF token with it. It
    // keeps the record's old id though, and the store only picks a new one on
    // a collision; since the CSRF middleware writes a new token before the
    // response leaves, the session would be recreated under the same id.
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
