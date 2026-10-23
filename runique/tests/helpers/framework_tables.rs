//! SQLite schema of the framework's tables (accounts, sessions, groups,
//! rights, history, reset tokens) for the integration tests. Shared by the
//! auth and form tests, with or without the admin feature.

// ── Schéma SQLite minimal (users + sessions + permissions + historique) ───────

#[cfg(feature = "pk-uuid")]
pub const USERS_DDL: &str = "
    CREATE TABLE eihwaz_users (
        id          BLOB PRIMARY KEY,
        username    TEXT NOT NULL UNIQUE,
        email       TEXT NOT NULL UNIQUE,
        password    TEXT NOT NULL,
        is_active   INTEGER NOT NULL DEFAULT 1,
        is_dirigeant INTEGER NOT NULL DEFAULT 0,
        is_admin    INTEGER NOT NULL DEFAULT 0,
        is_superuser INTEGER NOT NULL DEFAULT 0,
        created_at  TEXT,
        updated_at  TEXT,
        activated_at TEXT
    )
";

#[cfg(not(feature = "pk-uuid"))]
pub const USERS_DDL: &str = "
    CREATE TABLE eihwaz_users (
        id          INTEGER PRIMARY KEY AUTOINCREMENT,
        username    TEXT NOT NULL UNIQUE,
        email       TEXT NOT NULL UNIQUE,
        password    TEXT NOT NULL,
        is_active   INTEGER NOT NULL DEFAULT 1,
        is_dirigeant INTEGER NOT NULL DEFAULT 0,
        is_admin    INTEGER NOT NULL DEFAULT 0,
        is_superuser INTEGER NOT NULL DEFAULT 0,
        created_at  TEXT,
        updated_at  TEXT,
        activated_at TEXT
    )
";

#[cfg(feature = "pk-uuid")]
pub const SESSIONS_DDL: &str = "
    CREATE TABLE eihwaz_sessions (
        id          INTEGER PRIMARY KEY AUTOINCREMENT,
        cookie_id   TEXT NOT NULL UNIQUE,
        user_id     BLOB NOT NULL,
        session_id  TEXT NOT NULL,
        session_data TEXT,
        expires_at  TEXT NOT NULL
    )
";

#[cfg(not(feature = "pk-uuid"))]
pub const SESSIONS_DDL: &str = "
    CREATE TABLE eihwaz_sessions (
        id          INTEGER PRIMARY KEY AUTOINCREMENT,
        cookie_id   TEXT NOT NULL UNIQUE,
        user_id     INTEGER NOT NULL,
        session_id  TEXT NOT NULL,
        session_data TEXT,
        expires_at  TEXT NOT NULL
    )
";

#[cfg(feature = "pk-uuid")]
pub const GROUPES_DDL: &str = "
    CREATE TABLE eihwaz_groupes (
        id             INTEGER PRIMARY KEY AUTOINCREMENT,
        nom            TEXT NOT NULL,
        is_staff       INTEGER NOT NULL DEFAULT 0,
        is_active      INTEGER NOT NULL DEFAULT 1,
        responsable_id BLOB NOT NULL,
        repris_par     BLOB
    )
";

#[cfg(not(feature = "pk-uuid"))]
pub const GROUPES_DDL: &str = "
    CREATE TABLE eihwaz_groupes (
        id             INTEGER PRIMARY KEY AUTOINCREMENT,
        nom            TEXT NOT NULL,
        is_staff       INTEGER NOT NULL DEFAULT 0,
        is_active      INTEGER NOT NULL DEFAULT 1,
        responsable_id INTEGER NOT NULL,
        repris_par     INTEGER
    )
";

#[cfg(feature = "pk-uuid")]
pub const GROUPES_DROITS_DDL: &str = "
    CREATE TABLE eihwaz_groupes_droits (
        id             INTEGER PRIMARY KEY AUTOINCREMENT,
        groupe_id      INTEGER,
        user_id        BLOB,
        resource_key   TEXT NOT NULL,
        colonne        TEXT NOT NULL DEFAULT '',
        can_create     INTEGER NOT NULL DEFAULT 0,
        can_read       INTEGER NOT NULL DEFAULT 0,
        can_read_own   INTEGER NOT NULL DEFAULT 0,
        can_update     INTEGER NOT NULL DEFAULT 0,
        can_update_own INTEGER NOT NULL DEFAULT 0,
        can_delete     INTEGER NOT NULL DEFAULT 0,
        can_delete_own INTEGER NOT NULL DEFAULT 0
    )
";

#[cfg(not(feature = "pk-uuid"))]
pub const GROUPES_DROITS_DDL: &str = "
    CREATE TABLE eihwaz_groupes_droits (
        id             INTEGER PRIMARY KEY AUTOINCREMENT,
        groupe_id      INTEGER,
        user_id        INTEGER,
        resource_key   TEXT NOT NULL,
        colonne        TEXT NOT NULL DEFAULT '',
        can_create     INTEGER NOT NULL DEFAULT 0,
        can_read       INTEGER NOT NULL DEFAULT 0,
        can_read_own   INTEGER NOT NULL DEFAULT 0,
        can_update     INTEGER NOT NULL DEFAULT 0,
        can_update_own INTEGER NOT NULL DEFAULT 0,
        can_delete     INTEGER NOT NULL DEFAULT 0,
        can_delete_own INTEGER NOT NULL DEFAULT 0
    )
";

#[cfg(feature = "pk-uuid")]
pub const USERS_GROUPES_DDL: &str = "
    CREATE TABLE eihwaz_users_groupes (
        user_id   BLOB NOT NULL,
        groupe_id INTEGER NOT NULL,
        PRIMARY KEY (user_id, groupe_id)
    )
";

#[cfg(not(feature = "pk-uuid"))]
pub const USERS_GROUPES_DDL: &str = "
    CREATE TABLE eihwaz_users_groupes (
        user_id   INTEGER NOT NULL,
        groupe_id INTEGER NOT NULL,
        PRIMARY KEY (user_id, groupe_id)
    )
";

#[cfg(feature = "pk-uuid")]
pub const HISTORY_DDL: &str = "
    CREATE TABLE eihwaz_history (
        id            INTEGER PRIMARY KEY AUTOINCREMENT,
        resource_key  TEXT NOT NULL,
        object_pk     TEXT NOT NULL,
        action        TEXT NOT NULL,
        user_id       BLOB NOT NULL,
        username      TEXT NOT NULL,
        created_at    TEXT NOT NULL,
        summary       TEXT,
        batch_id      TEXT
    )
";

#[cfg(not(feature = "pk-uuid"))]
pub const HISTORY_DDL: &str = "
    CREATE TABLE eihwaz_history (
        id            INTEGER PRIMARY KEY AUTOINCREMENT,
        resource_key  TEXT NOT NULL,
        object_pk     TEXT NOT NULL,
        action        TEXT NOT NULL,
        user_id       INTEGER NOT NULL,
        username      TEXT NOT NULL,
        created_at    TEXT NOT NULL,
        summary       TEXT,
        batch_id      TEXT
    )
";

// Table du flux reset-password (admin création d'user + reset manuel) — cf.
// `tests/utils/test_reset_token.rs` pour le même schéma en isolation.
#[cfg(feature = "pk-uuid")]
pub const RESET_TOKENS_DDL: &str = "
    CREATE TABLE eihwaz_reset_tokens (
        id          INTEGER PRIMARY KEY AUTOINCREMENT,
        token_hash  TEXT NOT NULL UNIQUE,
        user_id     BLOB NOT NULL,
        expires_at  TEXT NOT NULL
    )
";

#[cfg(not(feature = "pk-uuid"))]
pub const RESET_TOKENS_DDL: &str = "
    CREATE TABLE eihwaz_reset_tokens (
        id          INTEGER PRIMARY KEY AUTOINCREMENT,
        token_hash  TEXT NOT NULL UNIQUE,
        user_id     INTEGER NOT NULL,
        expires_at  TEXT NOT NULL
    )
";
