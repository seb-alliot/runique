//! The constants of authentication: the resource keys of the framework's
//! tables, and the session keys of a signed-in account.

/// Resource keys of the framework's tables: the key of their `ModelMeta` and
/// of their rows in `eihwaz_groupes_droits`.
pub mod resource_key {
    /// Accounts (`eihwaz_users`).
    pub const USERS: &str = "users";
    /// Groups (`eihwaz_groupes`).
    pub const GROUPES: &str = "groupes";
    /// Rights (`eihwaz_groupes_droits`).
    pub const DROITS: &str = "droits";
    /// Memberships (`eihwaz_users_groupes`).
    pub const USERS_GROUPES: &str = "users_groupes";

    /// A group's row never targets one of these: the database refuses it
    /// (CHECK). Changing this list on a live base needs a migration.
    pub const FRAMEWORK_KEYS: &[&str] = &[USERS, GROUPES, DROITS, USERS_GROUPES];
}

/// Session key storing the authenticated user's id.
pub const SESSION_USER_ID_KEY: &str = "user_id";
/// Session key storing the account's `session_version` at sign-in: the
/// session ends once the account's has moved on.
pub const SESSION_VERSION_KEY: &str = "session_version";
/// Session key storing the Unix timestamp until which an anonymous session
/// is protected from cleanup. Set manually by the app for anonymous sessions
/// carrying value (cart, multi-step form) — the session cleaner skips any
/// session where this timestamp is still in the future.
pub const SESSION_ACTIVE_KEY: &str = "session_active";
