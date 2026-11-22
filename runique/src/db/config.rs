//! Database configuration and connection management.
//!
//! This module provides flexible configuration for connecting to different
//! databases (PostgreSQL, MySQL, SQLite) via SeaORM.
#![doc = include_str!("../../doc-tests/db/db_config_module.md")]

use dotenvy::dotenv;
use sea_orm::{ConnectOptions, Database, DatabaseConnection, DbErr, RuntimeErr};
use serde::{Deserialize, Serialize};
use std::env;
use std::time::Duration;

use super::builder::DatabaseConfigBuilder;
use super::engine::{DatabaseEngine, verify_database_driver};
use crate::utils::trad::{t, tf};

/// Advanced database configuration.
///
/// Contains all parameters needed to establish and manage a database
/// connection, including connection pools and timeouts.
#[doc = include_str!("../../doc-tests/db/db_config_advanced.md")]
#[derive(Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    /// Database connection URL
    // Holds the password in plain text, so it's never serialized and the hand-written `Debug` masks it.
    #[serde(skip_serializing)]
    pub url: String,
    /// Database type (PostgreSQL, MySQL, SQLite)
    pub engine: DatabaseEngine,
    /// Maximum number of connections in the pool
    pub max_connections: u32,
    /// Minimum number of connections in the pool
    pub min_connections: u32,
    /// Timeout for establishing a connection
    pub connect_timeout: Duration,
    /// Timeout for acquiring a connection from the pool
    pub acquire_timeout: Duration,
    /// Idle duration before closing a connection
    pub idle_timeout: Duration,
    /// Maximum lifetime of a connection
    pub max_lifetime: Duration,
    /// Enable or disable SQL logging
    pub sqlx_logging: bool,
    /// What [`connect`](Self::connect) falls back on when this address doesn't
    /// answer: the configuration the `DB_*` keys give, or why they give none.
    /// Only set when the address comes from `DATABASE_URL`.
    #[serde(skip)]
    pub(crate) fallback: Option<Result<Box<DatabaseConfig>, String>>,
    /// What [`connect`](Self::connect) tells the developer about the way this
    /// configuration was chosen.
    #[serde(skip)]
    pub(crate) notice: Option<Notice>,
}

/// Why a configuration is not the one `DATABASE_URL` asked for.
#[derive(Clone, Debug, PartialEq)]
pub(crate) enum Notice {
    /// `DATABASE_URL` is incomplete (the reason, as a translation key): the
    /// `DB_*` keys are used instead.
    UrlIncomplete(&'static str),
    /// `DATABASE_URL` asks for SQLite without naming a file: the default one.
    SqliteDefaultFile,
}

/// SQLite's file when nothing names one.
const DEFAULT_SQLITE_FILE: &str = "local_base.sqlite";

/// What `DATABASE_URL` gives.
enum UrlState {
    Absent,
    Complete(String),
    /// The reason, as a translation key.
    Incomplete(&'static str),
    /// `sqlite://` without a file: the engine is known, only the name misses.
    SqliteWithoutFile,
}

/// What the `DB_*` keys give.
enum KeysState {
    /// None of them is set.
    Empty,
    /// Enough to connect: the address they build.
    Filled(String),
    /// Some are set and they are not enough: the message that says why.
    Unusable(String),
}

/// A variable that is set but blank counts as absent: `DB_USER=` would
/// otherwise build `postgres://:@localhost…`.
fn present(get: &impl Fn(&str) -> Option<String>, key: &str) -> Option<String> {
    get(key).filter(|value| !value.trim().is_empty())
}

/// Reads `DATABASE_URL`. An address is incomplete in four cases only: a start
/// that names no engine, one that can't be read, a server without a database
/// name, SQLite without a file. Anything else is tried as it is, and the
/// server decides.
fn url_state(get: &impl Fn(&str) -> Option<String>) -> UrlState {
    let Some(url) = present(get, "DATABASE_URL") else {
        return UrlState::Absent;
    };
    let Ok(engine) = DatabaseEngine::detect_from_url(&url) else {
        return UrlState::Incomplete("database.reason_unknown_start");
    };
    if engine == DatabaseEngine::SQLite {
        let rest = url
            .strip_prefix("sqlite://")
            .or_else(|| url.strip_prefix("sqlite:"))
            .unwrap_or_default();
        let (file, query) = rest.split_once('?').unwrap_or((rest, ""));
        // No file, unless the base is asked for in memory: SQLite would open
        // a temporary one, gone when it closes.
        if file.is_empty() && !query.split('&').any(|param| param == "mode=memory") {
            return UrlState::SqliteWithoutFile;
        }
        return UrlState::Complete(url);
    }
    let Ok(parsed) = url::Url::parse(&url) else {
        return UrlState::Incomplete("database.reason_unreadable");
    };
    let named_in_path = !parsed.path().trim_matches('/').is_empty();
    // PostgreSQL also takes the database as a parameter.
    let named_in_query = engine == DatabaseEngine::PostgreSQL
        && parsed
            .query_pairs()
            .any(|(key, value)| key == "dbname" && !value.is_empty());
    if !named_in_path && !named_in_query {
        return UrlState::Incomplete("database.reason_no_database");
    }
    UrlState::Complete(url)
}

/// Reads the `DB_*` keys and builds their address.
fn keys_state(get: &impl Fn(&str) -> Option<String>) -> KeysState {
    let Some(keyword) = present(get, "DB_ENGINE") else {
        let set: Vec<&str> = ["DB_USER", "DB_PASSWORD", "DB_HOST", "DB_PORT", "DB_NAME"]
            .into_iter()
            .filter(|key| present(get, key).is_some())
            .collect();
        if set.is_empty() {
            return KeysState::Empty;
        }
        // Without the engine no address can be built, and SQLite must not be
        // taken in its place.
        return KeysState::Unusable(tf("database.keys_without_engine", &[set.join(", ")]));
    };
    let Some(engine) = DatabaseEngine::from_keyword(&keyword) else {
        return KeysState::Unusable(format!(
            " Unsupported DB_ENGINE: {}\n\nSupported engines: postgres, mysql, sqlite\nOr set DATABASE_URL directly.",
            keyword.trim().to_ascii_lowercase()
        ));
    };
    let Some(default_port) = engine.default_port() else {
        // SQLite: a file.
        let name = present(get, "DB_NAME").unwrap_or_else(|| DEFAULT_SQLITE_FILE.to_string());
        return KeysState::Filled(format!("{}://{}?mode=rwc", engine.scheme(), name));
    };
    // A server: an account, a host and a port.
    let Some(user) = present(get, "DB_USER") else {
        return KeysState::Unusable(format!(" DB_USER not set for {}\n\nRequired variables:\n  - DB_USER\n  - DB_PASSWORD\n  - DB_HOST (optional, default: localhost)\n  - DB_PORT (optional, default: {})\n  - DB_NAME", engine.name(), default_port));
    };
    let Some(password) = present(get, "DB_PASSWORD") else {
        return KeysState::Unusable(format!(" DB_PASSWORD not set for {}", engine.name()));
    };
    let Some(name) = present(get, "DB_NAME") else {
        return KeysState::Unusable(format!(" DB_NAME not set for {}", engine.name()));
    };
    let host = present(get, "DB_HOST").unwrap_or_else(|| "localhost".to_string());
    let port = present(get, "DB_PORT").unwrap_or_else(|| default_port.to_string());
    // Written as they are in the `.env`, encoded here: a `/`, `@` or `:` left
    // raw would cut the address elsewhere, or name another host.
    KeysState::Filled(format!(
        "{}://{}:{}@{}:{}/{}",
        engine.scheme(),
        urlencoding::encode(&user),
        urlencoding::encode(&password),
        host,
        port,
        name
    ))
}

impl DatabaseConfig {
    /// Creates a configuration from a connection URL.
    ///
    /// The database type is automatically detected from the URL.
    /// Default values are used for other parameters.
    ///
    /// # Arguments
    ///
    /// * `url` - Connection URL (e.g., "sqlite://db.sqlite", "postgres://user:pass@host/db")
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use runique::prelude::DatabaseConfig;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// // SQLite
    /// let config = DatabaseConfig::from_url("sqlite://app.db")?.build();
    ///
    /// // PostgreSQL
    /// let config = DatabaseConfig::from_url("postgres://user:pass@localhost/mydb")?
    ///     .max_connections(100)
    ///     .build();
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if the URL is invalid or unsupported.
    pub fn from_url(url: impl Into<String>) -> Result<DatabaseConfigBuilder, String> {
        let url = url.into();
        let engine = DatabaseEngine::detect_from_url(&url)?;

        Ok(DatabaseConfigBuilder {
            config: DatabaseConfig {
                url,
                engine,
                // Small on purpose: a first connection to a server can take
                // longer than half a second in a debug build, and one site
                // must not hold the whole server (100 connections by default
                // on PostgreSQL). A busy site raises them in its `.env`.
                max_connections: 10,
                min_connections: 2,
                connect_timeout: Duration::from_secs(2),
                acquire_timeout: Duration::from_secs(5),
                idle_timeout: Duration::from_secs(300),
                max_lifetime: Duration::from_secs(3600),
                sqlx_logging: false,
                fallback: None,
                notice: None,
            },
        })
    }

    /// Creates a configuration from environment variables.
    ///
    /// Reads configuration from a `.env` file or system environment variables.
    ///
    /// # Environment Variables
    ///
    /// **Connection (URL directe — prioritaire)**
    /// - `DATABASE_URL` - Full connection URL (e.g., `postgres://user:pass@host/db`). Takes priority over all component variables. Compatible with `sea-orm-cli`.
    ///
    /// **Connection (variables composantes)**
    /// - `DB_ENGINE` - Database type: `postgres`, `mysql`, `sqlite` (default: `sqlite`)
    /// - `DB_USER` - Username (required for non-SQLite)
    /// - `DB_PASSWORD` - Password (required for non-SQLite)
    /// - `DB_HOST` - Host (default: `localhost`)
    /// - `DB_PORT` - Port (default: `5432` for PostgreSQL, `3306` for MySQL)
    /// - `DB_NAME` - Database name (default: `local_base.sqlite` for SQLite)
    ///
    /// **Connection pool**
    /// - `DB_MAX_CONNECTIONS` - Maximum pool size (default: `10`)
    /// - `DB_MIN_CONNECTIONS` - Minimum pool size (default: `2`)
    ///
    /// **Timeouts (in seconds)**
    /// - `DB_CONNECT_TIMEOUT` - Connection timeout (default: `2`)
    /// - `DB_ACQUIRE_TIMEOUT` - Pool acquire timeout in milliseconds (default: `5000`)
    /// - `DB_IDLE_TIMEOUT` - Idle connection lifetime (default: `300`)
    /// - `DB_MAX_LIFETIME` - Maximum connection lifetime (default: `3600`)
    ///
    /// **Logging**
    /// - `DB_LOGGING` - Enable SQL query logging: `true` / `false` (default: `false`)
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use runique::prelude::DatabaseConfig;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let config = DatabaseConfig::from_env()?.build();
    /// let db = config.connect().await?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if required variables are missing.
    pub fn from_env() -> Result<DatabaseConfigBuilder, String> {
        dotenv().ok();
        Self::from_lookup(|key| env::var(key).ok())
    }

    /// Same as [`from_env`](Self::from_env), but reads each variable through
    /// `get` instead of the process environment. The test builder relies on it
    /// to read an env file without modifying the environment, which isn't safe
    /// while other test threads may be reading it.
    ///
    /// # Errors
    ///
    /// Returns an error if required variables are missing.
    pub fn from_lookup(
        get: impl Fn(&str) -> Option<String>,
    ) -> Result<DatabaseConfigBuilder, String> {
        // One rule for every caller. `DATABASE_URL` is read first; the
        // `DB_*` keys take over when it is absent or incomplete, and stand
        // by as a fallback when it is complete.
        let (url, fallback, notice) = match (url_state(&get), keys_state(&get)) {
            (UrlState::Complete(url), KeysState::Filled(keys)) => (url, Some(Ok(keys)), None),
            (UrlState::Complete(url), KeysState::Unusable(why)) => (url, Some(Err(why)), None),
            (UrlState::Complete(url), KeysState::Empty) => (url, None, None),
            (UrlState::Incomplete(reason), KeysState::Filled(keys)) => {
                (keys, None, Some(Notice::UrlIncomplete(reason)))
            }
            (UrlState::SqliteWithoutFile, KeysState::Filled(keys)) => (
                keys,
                None,
                Some(Notice::UrlIncomplete("database.reason_sqlite_no_file")),
            ),
            (UrlState::Incomplete(reason), KeysState::Unusable(why)) => {
                return Err(tf("database.url_incomplete_keys_unusable", &[t(reason).into_owned(), why]));
            }
            (UrlState::SqliteWithoutFile, KeysState::Unusable(why)) => {
                return Err(tf(
                    "database.url_incomplete_keys_unusable",
                    &[t("database.reason_sqlite_no_file").into_owned(), why],
                ));
            }
            // No SQLite by default after an address that aimed at a server.
            (UrlState::Incomplete(reason), KeysState::Empty) => {
                return Err(tf("database.url_incomplete_no_keys", &[t(reason)]));
            }
            // The address itself asked for SQLite: only the file name misses.
            (UrlState::SqliteWithoutFile, KeysState::Empty) => (
                format!("sqlite://{DEFAULT_SQLITE_FILE}?mode=rwc"),
                None,
                Some(Notice::SqliteDefaultFile),
            ),
            (UrlState::Absent, KeysState::Filled(keys)) => (keys, None, None),
            (UrlState::Absent, KeysState::Unusable(why)) => return Err(why),
            // Nothing set at all: SQLite, as the documentation says.
            (UrlState::Absent, KeysState::Empty) => {
                (format!("sqlite://{DEFAULT_SQLITE_FILE}?mode=rwc"), None, None)
            }
        };

        let mut builder = Self::from_url(url)?;

        if let Some(v) = get("DB_MAX_CONNECTIONS")
            && let Ok(n) = v.parse::<u32>()
        {
            builder.config.max_connections = n;
        }
        if let Some(v) = get("DB_MIN_CONNECTIONS")
            && let Ok(n) = v.parse::<u32>()
        {
            builder.config.min_connections = n;
        }
        if let Some(v) = get("DB_CONNECT_TIMEOUT")
            && let Ok(n) = v.parse::<u64>()
        {
            builder.config.connect_timeout = Duration::from_secs(n);
        }
        if let Some(v) = get("DB_ACQUIRE_TIMEOUT")
            && let Ok(n) = v.parse::<u64>()
        {
            builder.config.acquire_timeout = Duration::from_millis(n);
        }
        if let Some(v) = get("DB_IDLE_TIMEOUT")
            && let Ok(n) = v.parse::<u64>()
        {
            builder.config.idle_timeout = Duration::from_secs(n);
        }
        if let Some(v) = get("DB_MAX_LIFETIME")
            && let Ok(n) = v.parse::<u64>()
        {
            builder.config.max_lifetime = Duration::from_secs(n);
        }
        if let Some(v) = get("DB_LOGGING") {
            builder.config.sqlx_logging = crate::utils::config::env::flag_from(&v).unwrap_or(false);
        }

        // The fallback connects with the same pool and timeouts.
        builder.config.fallback = fallback.map(|keys| {
            keys.and_then(|url| {
                let mut config = Self::from_url(url)?.build();
                config.max_connections = builder.config.max_connections;
                config.min_connections = builder.config.min_connections;
                config.connect_timeout = builder.config.connect_timeout;
                config.acquire_timeout = builder.config.acquire_timeout;
                config.idle_timeout = builder.config.idle_timeout;
                config.max_lifetime = builder.config.max_lifetime;
                config.sqlx_logging = builder.config.sqlx_logging;
                Ok(Box::new(config))
            })
        });
        builder.config.notice = notice;

        Ok(builder)
    }

    /// Establishes a connection to the database.
    ///
    /// Creates a connection pool configured according to this `DatabaseConfig`.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use runique::prelude::DatabaseConfig;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let config = DatabaseConfig::from_env()?.build();
    /// let db = config.connect().await?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// - Returns an error if the driver is not enabled (missing Cargo feature)
    /// - Returns an error if the connection fails
    pub async fn connect(&self) -> Result<DatabaseConnection, DbErr> {
        self.connect_resolved().await.map(|(conn, _)| conn)
    }

    /// The same pool size for this configuration and its fallback: a command
    /// or a test that only ever needs one connection.
    pub(crate) fn pool(&mut self, max: u32, min: u32) {
        self.max_connections = max;
        self.min_connections = min;
        if let Some(Ok(keys)) = &mut self.fallback {
            keys.max_connections = max;
            keys.min_connections = min;
        }
    }

    /// Connects, and says which configuration answered: this one, or the one
    /// of the `DB_*` keys when the address of `DATABASE_URL` didn't (the two
    /// may name the same database: the second try then tells whether the
    /// failure came from the address). The developer is told on the console.
    pub(crate) async fn connect_resolved(
        &self,
    ) -> Result<(DatabaseConnection, &DatabaseConfig), DbErr> {
        match &self.notice {
            Some(Notice::UrlIncomplete(reason)) => {
                eprintln!("{}", tf("database.url_incomplete_using_keys", &[t(reason)]));
            }
            Some(Notice::SqliteDefaultFile) => {
                eprintln!("{}", tf("database.url_sqlite_default_file", &[DEFAULT_SQLITE_FILE]));
            }
            None => {}
        }
        let first = match self.connect_once().await {
            Ok(conn) => {
                if let Some(Notice::UrlIncomplete(reason)) = &self.notice {
                    eprintln!("{}", tf("database.url_incomplete_reminder", &[t(reason)]));
                }
                return Ok((conn, self));
            }
            Err(error) => error,
        };
        match &self.fallback {
            None => Err(first),
            Some(Err(why)) => Err(DbErr::Custom(tf(
                "database.url_failed_keys_unusable",
                &[first.to_string(), why.clone()],
            ))),
            Some(Ok(keys)) => {
                eprintln!("{}", tf("database.url_failed_trying_keys", &[&first]));
                let conn = keys.connect_once().await?;
                Ok((conn, keys.as_ref()))
            }
        }
    }

    async fn connect_once(&self) -> Result<DatabaseConnection, DbErr> {
        if let Some(level) = crate::utils::runique_log::log_level!(db.connect) {
            crate::runique_log!(level, "  Connecting to {} database...", self.engine.name());
        }

        verify_database_driver(&self.engine).map_err(DbErr::Custom)?;

        let mut opt = ConnectOptions::new(&self.url);

        match self.engine {
            DatabaseEngine::SQLite => {
                opt.max_connections(1).min_connections(1);
            }
            _ => {
                opt.max_connections(self.max_connections)
                    .min_connections(self.min_connections);
            }
        }

        opt.connect_timeout(self.connect_timeout)
            .acquire_timeout(self.acquire_timeout)
            .idle_timeout(self.idle_timeout)
            .max_lifetime(self.max_lifetime)
            .sqlx_logging(self.sqlx_logging)
            .sqlx_logging_level(tracing::log::LevelFilter::Off);

        match Database::connect(opt).await {
            Ok(conn) => {
                if let Some(level) = crate::utils::runique_log::log_level!(db.connect) {
                    crate::runique_log!(
                        level,
                        "Database connected successfully ({})",
                        self.engine.name()
                    );
                }
                Ok(conn)
            }
            Err(e) => {
                tracing::error!("└──> Database connection failed");
                tracing::error!("   └──> Engine: {}", self.engine.name());
                tracing::error!("       └──> URL: {}", mask_password(&self.url));
                Err(redact_url(e, &self.url))
            }
        }
    }
}

impl DatabaseConfig {
    /// The name of the database this configuration connects to, read from its
    /// URL: what follows the host for a server (or `?dbname=` on PostgreSQL),
    /// the file for SQLite. `None` when the URL names none, or when it can be
    /// read two ways (see [`database_name`]).
    pub(crate) fn database_name(&self) -> Option<String> {
        database_name(&self.url, &self.engine)
    }

    /// The name the startup message shows: the one of the configuration the
    /// environment gives, read the way [`from_env`](Self::from_env) connects
    /// (`DATABASE_URL` first, then the `DB_*` keys, then SQLite's default
    /// file). `None` when the environment gives no usable configuration.
    pub(crate) fn displayed_name() -> Option<String> {
        Self::from_env().ok()?.build().database_name()
    }
}

/// The database a connection URL names, for display only.
///
/// The user and the password sit before the last `@` and the query may hold
/// a `password=`: neither must come out. A raw `?` or `@` inside a secret
/// makes the URL readable two ways, so the name is read both ways (`@` first,
/// then `?` first) and shown only when they agree.
fn database_name(url: &str, engine: &DatabaseEngine) -> Option<String> {
    let (_, rest) = url.split_once(':')?;
    let after_at =
        |s: &'_ str| -> String { s.rsplit_once('@').map_or(s, |(_, after)| after).to_string() };
    let before_query = |s: &'_ str| -> String {
        s.split_once('?')
            .map_or(s, |(before, _)| before)
            .to_string()
    };
    let name_of = |location: String| -> String {
        let location = location.trim_start_matches("//");
        match engine {
            DatabaseEngine::SQLite => location.to_string(),
            _ => location
                .split_once('/')
                .map_or("", |(_, name)| name)
                .to_string(),
        }
    };

    let name = name_of(before_query(&after_at(rest)));
    if name != name_of(after_at(&before_query(rest))) {
        return None;
    }
    if !name.is_empty() {
        return Some(name);
    }
    if !matches!(engine, DatabaseEngine::PostgreSQL) {
        return None;
    }
    let (_, query) = rest.split_once('?')?;
    query
        .split('&')
        .find_map(|param| param.strip_prefix("dbname="))
        .filter(|name| !name.is_empty())
        .map(String::from)
}

impl std::fmt::Debug for DatabaseConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DatabaseConfig")
            .field("url", &mask_password(&self.url))
            .field("engine", &self.engine)
            .field("max_connections", &self.max_connections)
            .field("min_connections", &self.min_connections)
            .field("connect_timeout", &self.connect_timeout)
            .field("acquire_timeout", &self.acquire_timeout)
            .field("idle_timeout", &self.idle_timeout)
            .field("max_lifetime", &self.max_lifetime)
            .field("sqlx_logging", &self.sqlx_logging)
            .finish()
    }
}

/// Hides the password in a connection URL so it's safe to log: the one in
/// `user:password@`, and any passed as a query parameter (`?password=…`).
pub(crate) fn mask_password(url: &str) -> String {
    mask_query_secrets(&mask_userinfo(url))
}

fn mask_userinfo(url: &str) -> String {
    let Some(idx) = url.find("://") else {
        return url.to_string();
    };
    let protocol_end = idx.checked_add(3).filter(|&x| x <= url.len());
    let Some(after_protocol) = protocol_end else {
        return url.to_string();
    };
    // Split on the last `@`, not the first: hosts never contain one, but a raw password might.
    let Some(at_idx) = url[after_protocol..].rfind('@') else {
        return url.to_string();
    };
    let at_pos = after_protocol.saturating_add(at_idx);
    let before = &url[..after_protocol];
    let after = &url[at_pos..];
    let creds = &url[after_protocol..at_pos];
    let Some(colon) = creds.find(':') else {
        return url.to_string();
    };
    let user = &creds[..colon];
    format!("{}{}:****{}", before, user, after)
}

/// SeaORM quotes the full connection string in some errors (e.g. "has no
/// supporting driver"), which then reach the terminal through `?`.
pub(crate) fn redact_url(err: DbErr, url: &str) -> DbErr {
    match err {
        DbErr::Conn(RuntimeErr::Internal(msg)) if msg.contains(url) => {
            DbErr::Conn(RuntimeErr::Internal(msg.replace(url, &mask_password(url))))
        }
        other => other,
    }
}

/// libpq, MySQL and sqlx all accept the password as a query parameter too.
fn mask_query_secrets(url: &str) -> String {
    const SECRET_KEYS: [&str; 5] = ["password", "pass", "pwd", "passwd", "sslpassword"];
    let Some((base, query)) = url.split_once('?') else {
        return url.to_string();
    };
    let params: Vec<String> = query
        .split('&')
        .map(|param| match param.split_once('=') {
            Some((key, _)) if SECRET_KEYS.contains(&key.to_ascii_lowercase().as_str()) => {
                format!("{key}=****")
            }
            _ => param.to_string(),
        })
        .collect();
    format!("{base}?{}", params.join("&"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_redact_url_hides_the_password_in_a_connection_error() {
        let url = "postgresql://postgres:secret123@localhost:5432/db";
        let err = DbErr::Conn(RuntimeErr::Internal(format!(
            "The connection string '{url}' has no supporting driver."
        )));
        let shown = redact_url(err, url).to_string();
        assert!(!shown.contains("secret123"), "{shown}");
        assert!(
            shown.contains("postgresql://postgres:****@localhost:5432/db"),
            "{shown}"
        );
    }

    #[test]
    fn test_redact_url_leaves_other_errors_alone() {
        let err = DbErr::Conn(RuntimeErr::Internal("timeout".to_string()));
        assert_eq!(
            redact_url(err, "postgres://u:p@h/db").to_string(),
            "Connection Error: timeout"
        );
    }

    #[test]
    fn test_mask_password() {
        let url = "postgres://myuser:secret123@localhost:5432/mydb";
        let masked = mask_password(url);
        assert_eq!(masked, "postgres://myuser:****@localhost:5432/mydb");
    }

    #[test]
    fn test_mask_password_no_password() {
        let url = "sqlite://local.db";
        let masked = mask_password(url);
        assert_eq!(masked, "sqlite://local.db");
    }

    #[test]
    fn test_mask_password_with_at_in_password() {
        let url = "postgres://myuser:p@ss@localhost:5432/mydb";
        let masked = mask_password(url);
        assert_eq!(masked, "postgres://myuser:****@localhost:5432/mydb");
    }

    #[test]
    fn test_mask_password_in_query_parameter() {
        let masked = mask_password(
            "postgres://db.example.com/app?user=bob&password=hunter2&sslmode=require",
        );
        assert!(!masked.contains("hunter2"), "{masked}");
        assert!(masked.contains("password=****"));
        assert!(masked.contains("sslmode=require"));
    }

    #[test]
    fn test_mask_password_in_both_places() {
        let masked = mask_password("mysql://bob:secret1@db:3306/app?PWD=secret2");
        assert!(
            !masked.contains("secret1") && !masked.contains("secret2"),
            "{masked}"
        );
    }

    #[test]
    fn test_debug_masks_password() {
        let config = DatabaseConfig::from_url("postgres://myuser:secret123@localhost:5432/mydb")
            .unwrap()
            .build();
        let debug = format!("{config:?}");
        assert!(!debug.contains("secret123"));
        assert!(debug.contains("myuser:****@localhost"));
    }

    fn name_of(url: &str) -> Option<String> {
        DatabaseConfig::from_url(url)
            .unwrap()
            .build()
            .database_name()
    }

    #[test]
    fn test_database_name_of_a_server_url() {
        assert_eq!(
            name_of("postgres://bob:secret@localhost:5432/shop").as_deref(),
            Some("shop")
        );
        assert_eq!(
            name_of("mysql://bob:secret@db:3306/shop?ssl-mode=required").as_deref(),
            Some("shop")
        );
        assert_eq!(
            name_of("postgres://localhost/shop").as_deref(),
            Some("shop")
        );
    }

    #[test]
    fn test_database_name_absent_from_a_server_url() {
        assert_eq!(name_of("postgres://bob:secret@localhost:5432"), None);
        assert_eq!(name_of("postgres://bob:secret@localhost:5432/"), None);
        assert_eq!(
            name_of("mysql://bob:secret@db:3306/?ssl-mode=required"),
            None
        );
    }

    #[test]
    fn test_database_name_from_the_dbname_parameter_on_postgres_only() {
        assert_eq!(
            name_of("postgres://db.local?user=bob&dbname=shop").as_deref(),
            Some("shop")
        );
        assert_eq!(name_of("postgres://db.local?user=bob&dbname="), None);
        assert_eq!(name_of("mysql://db.local?dbname=shop"), None);
    }

    #[test]
    fn test_database_name_of_sqlite_is_its_file() {
        assert_eq!(
            name_of("sqlite://data.db?mode=rwc").as_deref(),
            Some("data.db")
        );
        assert_eq!(name_of("sqlite://:memory:").as_deref(), Some(":memory:"));
        assert_eq!(name_of("sqlite://"), None);
    }

    #[test]
    fn test_database_name_never_shows_a_secret() {
        // `@` in the password: still read after the last one.
        assert_eq!(
            name_of("postgres://bob:p@ss/word@localhost/shop").as_deref(),
            Some("shop")
        );
        // Raw `?` and `/` in the password: `?` first would show `word`.
        assert_eq!(name_of("postgres://bob:pass/word?x@localhost/shop"), None);
        // `@` and `/` in a query password: `@` first would show `er2`.
        assert_eq!(name_of("postgres://db.local/shop?password=hu@nt/er2"), None);
        for url in [
            "postgres://bob:secret123@localhost/shop",
            "postgres://db.local/shop?user=bob&password=secret123",
            "postgres://bob:secret123@localhost",
        ] {
            let shown = name_of(url).unwrap_or_default();
            assert!(
                !shown.contains("secret123") && !shown.contains("bob"),
                "{shown}"
            );
        }
    }

    #[test]
    fn test_database_name_follows_the_url_built_from_components() {
        let vars: std::collections::HashMap<&str, &str> = [
            ("DB_ENGINE", "postgres"),
            ("DB_USER", "bob"),
            ("DB_PASSWORD", "secret"),
            ("DB_NAME", "app"),
        ]
        .into();
        let config = DatabaseConfig::from_lookup(|key| vars.get(key).map(|v| v.to_string()))
            .unwrap()
            .build();
        assert_eq!(config.database_name().as_deref(), Some("app"));

        // Nothing set: SQLite's default file, the one actually opened.
        let config = DatabaseConfig::from_lookup(|_| None).unwrap().build();
        assert_eq!(config.database_name().as_deref(), Some("local_base.sqlite"));
    }

    #[test]
    fn test_from_lookup_builds_url_from_components() {
        let vars: std::collections::HashMap<&str, &str> = [
            ("DB_ENGINE", "postgres"),
            ("DB_USER", "bob"),
            ("DB_PASSWORD", "secret"),
            ("DB_NAME", "app"),
        ]
        .into();
        let config = DatabaseConfig::from_lookup(|key| vars.get(key).map(|v| v.to_string()))
            .unwrap()
            .build();
        assert_eq!(config.url, "postgres://bob:secret@localhost:5432/app");
    }

    #[test]
    fn test_from_lookup_database_url_wins() {
        let vars: std::collections::HashMap<&str, &str> = [
            ("DATABASE_URL", "sqlite://direct.db"),
            ("DB_ENGINE", "postgres"),
        ]
        .into();
        let config = DatabaseConfig::from_lookup(|key| vars.get(key).map(|v| v.to_string()))
            .unwrap()
            .build();
        assert_eq!(config.url, "sqlite://direct.db");
    }

    // ── The connection rule: `DATABASE_URL` first, the `DB_*` keys otherwise ──

    const SERVER_KEYS: [(&str, &str); 4] = [
        ("DB_ENGINE", "postgres"),
        ("DB_USER", "bob"),
        ("DB_PASSWORD", "secret"),
        ("DB_NAME", "app"),
    ];
    const KEYS_URL: &str = "postgres://bob:secret@localhost:5432/app";
    const DEFAULT_URL: &str = "sqlite://local_base.sqlite?mode=rwc";

    fn rule(vars: &[(&str, &str)]) -> Result<DatabaseConfig, String> {
        DatabaseConfig::from_lookup(|key| {
            vars.iter()
                .find(|(name, _)| *name == key)
                .map(|(_, value)| value.to_string())
        })
        .map(|builder| builder.build())
    }

    fn with_url<'a>(url: &'a str, keys: &[(&'a str, &'a str)]) -> Vec<(&'a str, &'a str)> {
        let mut vars = vec![("DATABASE_URL", url)];
        vars.extend_from_slice(keys);
        vars
    }

    /// The addresses that are complete: tried as they are.
    const COMPLETE: [&str; 11] = [
        "postgres://app:secret@db.local:5432/shop",
        "postgres://app:secret@db.local/shop",
        "postgres://app@db.local/shop",
        "postgres://db.local/shop",
        "postgres://db.local?dbname=shop&user=app",
        "mysql://app:secret@db.local/shop",
        "sqlite://data.db",
        "sqlite://data.db?mode=rwc",
        "sqlite:data.db",
        "sqlite::memory:",
        "sqlite://?mode=memory",
    ];

    /// The addresses that are incomplete, and why.
    const INCOMPLETE: [(&str, &str); 10] = [
        ("mariadb://app:secret@db.local/shop", "database.reason_unknown_start"),
        ("app:secret@db.local/shop", "database.reason_unknown_start"),
        ("\"postgres://app:secret@db.local/shop\"", "database.reason_unknown_start"),
        ("postgres://app:secret@db.local:543a/shop", "database.reason_unreadable"),
        // A raw `/` in the password: the host becomes `app`, the port `pa`.
        ("postgres://app:pa/ss@db.local/shop", "database.reason_unreadable"),
        ("postgres://app:secret@db.local", "database.reason_no_database"),
        ("postgres://app:secret@db.local/", "database.reason_no_database"),
        ("postgres://db.local?dbname=", "database.reason_no_database"),
        // `dbname` is a PostgreSQL parameter only.
        ("mysql://db.local?dbname=shop", "database.reason_no_database"),
        ("sqlite://", "database.reason_sqlite_no_file"),
    ];

    // Line 1: a complete address is used as it is. The keys stand by: their
    // address when they are enough, the reason when they are not, nothing
    // when none is set.
    #[test]
    fn a_complete_address_is_used_and_the_keys_stand_by() {
        for url in COMPLETE {
            let alone = rule(&with_url(url, &[])).unwrap();
            assert_eq!(alone.url, url);
            assert!(alone.fallback.is_none() && alone.notice.is_none(), "{url}");

            let backed = rule(&with_url(url, &SERVER_KEYS)).unwrap();
            assert_eq!(backed.url, url);
            assert_eq!(backed.fallback.unwrap().unwrap().url, KEYS_URL, "{url}");
            assert!(backed.notice.is_none(), "{url}");

            let partial = rule(&with_url(url, &SERVER_KEYS[..2])).unwrap();
            assert_eq!(partial.url, url);
            let why = partial.fallback.unwrap().unwrap_err();
            assert!(why.contains("DB_PASSWORD"), "{url}: {why}");
        }
    }

    // Line 2: an incomplete address with keys that are enough: the keys are
    // used, and the reason kept for the console. The fallback is the address
    // itself, so there is none.
    #[test]
    fn an_incomplete_address_gives_way_to_keys_that_are_enough() {
        for (url, reason) in INCOMPLETE {
            let config = rule(&with_url(url, &SERVER_KEYS)).unwrap();
            assert_eq!(config.url, KEYS_URL, "{url}");
            assert_eq!(config.notice, Some(Notice::UrlIncomplete(reason)), "{url}");
            assert!(config.fallback.is_none(), "{url}");
        }
        // The developer wrote SQLite in the keys: that is what is used.
        let sqlite = rule(&with_url("postgres://db.local", &[("DB_ENGINE", "sqlite")])).unwrap();
        assert_eq!(sqlite.url, DEFAULT_URL);
    }

    // Line 3: incomplete, and the keys are not enough: blocked, with the
    // reason and what is missing.
    #[test]
    fn an_incomplete_address_with_partial_keys_is_refused() {
        for (url, reason) in INCOMPLETE {
            let error = rule(&with_url(url, &SERVER_KEYS[..2])).unwrap_err();
            assert!(error.contains(t(reason).as_ref()), "{url}: {error}");
            assert!(error.contains("DB_PASSWORD"), "{url}: {error}");
        }
    }

    // Lines 4a and 4b: incomplete and no key at all. An address that aimed at
    // a server, or named no engine, never ends on SQLite; one that asked for
    // SQLite without a file gets the default file, and says so.
    #[test]
    fn an_incomplete_address_alone_is_refused_unless_it_asked_for_sqlite() {
        for (url, reason) in INCOMPLETE {
            if reason == "database.reason_sqlite_no_file" {
                continue;
            }
            assert_eq!(
                rule(&with_url(url, &[])).unwrap_err(),
                tf("database.url_incomplete_no_keys", &[t(reason)]),
                "{url}"
            );
        }
        for url in ["sqlite://", "sqlite:", "sqlite://?mode=rwc"] {
            let config = rule(&with_url(url, &[])).unwrap();
            assert_eq!(config.url, DEFAULT_URL, "{url}");
            assert_eq!(config.notice, Some(Notice::SqliteDefaultFile), "{url}");
        }
    }

    // Lines 5 to 7: no address. Keys that are enough build theirs, partial
    // ones are refused, and nothing at all is SQLite's default file.
    #[test]
    fn without_an_address_the_keys_decide() {
        let filled = rule(&SERVER_KEYS).unwrap();
        assert_eq!(filled.url, KEYS_URL);
        assert!(filled.fallback.is_none() && filled.notice.is_none());

        assert!(rule(&SERVER_KEYS[..3]).unwrap_err().contains("DB_NAME"));
        assert!(rule(&SERVER_KEYS[..1]).unwrap_err().contains("DB_USER"));
        assert!(
            rule(&[("DB_ENGINE", "oracle")])
                .unwrap_err()
                .contains("Unsupported DB_ENGINE: oracle")
        );

        let nothing = rule(&[]).unwrap();
        assert_eq!(nothing.url, DEFAULT_URL);
        assert!(nothing.fallback.is_none() && nothing.notice.is_none());
        assert_eq!(
            rule(&[("DB_ENGINE", "sqlite"), ("DB_NAME", "data.db")])
                .unwrap()
                .url,
            "sqlite://data.db?mode=rwc"
        );
    }

    // Keys without `DB_ENGINE`: no address can be built, and SQLite is not
    // taken in its place. Each key counts, and the message names it.
    #[test]
    fn keys_without_an_engine_are_refused() {
        for key in ["DB_USER", "DB_PASSWORD", "DB_HOST", "DB_PORT", "DB_NAME"] {
            assert_eq!(
                rule(&[(key, "x")]).unwrap_err(),
                tf("database.keys_without_engine", &[key]),
                "{key}"
            );
        }
        assert_eq!(
            rule(&[("DB_USER", "bob"), ("DB_NAME", "app")]).unwrap_err(),
            tf("database.keys_without_engine", &["DB_USER, DB_NAME"])
        );
        // With a complete address they only make the fallback unusable.
        let config = rule(&with_url("sqlite://data.db", &[("DB_USER", "bob")])).unwrap();
        assert!(config.fallback.unwrap().is_err());
    }

    // A key set but blank is absent, the address included.
    #[test]
    fn a_blank_value_is_an_absent_one() {
        assert_eq!(rule(&[("DATABASE_URL", "  ")]).unwrap().url, DEFAULT_URL);
        assert_eq!(rule(&[("DATABASE_URL", ""), ("DB_ENGINE", " ")]).unwrap().url, DEFAULT_URL);
        assert_eq!(
            rule(&with_url("", &SERVER_KEYS)).unwrap().url,
            KEYS_URL,
            "a blank address leaves the keys alone, without a notice"
        );
        let mut blank_password = SERVER_KEYS;
        blank_password[2] = ("DB_PASSWORD", "");
        assert!(rule(&blank_password).unwrap_err().contains("DB_PASSWORD"));
        // A blank engine with another key set is still a key without engine.
        assert!(rule(&[("DB_ENGINE", ""), ("DB_NAME", "app")]).is_err());
        // Optional keys left blank take their default.
        let mut blank_host = SERVER_KEYS.to_vec();
        blank_host.extend([("DB_HOST", " "), ("DB_PORT", "")]);
        assert_eq!(rule(&blank_host).unwrap().url, KEYS_URL);
    }

    // The account written as it is in the `.env` is encoded in the address
    // built from the keys, and read back unchanged.
    #[test]
    fn the_account_of_the_keys_is_encoded() {
        let mut keys = SERVER_KEYS;
        keys[1] = ("DB_USER", "bo b@x");
        keys[2] = ("DB_PASSWORD", "p@ss/w:rd#1?");
        let config = rule(&keys).unwrap();

        assert_eq!(
            config.url,
            "postgres://bo%20b%40x:p%40ss%2Fw%3Ard%231%3F@localhost:5432/app"
        );
        let parsed = url::Url::parse(&config.url).unwrap();
        assert_eq!(parsed.host_str(), Some("localhost"));
        assert_eq!(parsed.path(), "/app");
        assert_eq!(
            urlencoding::decode(parsed.username()).unwrap(),
            "bo b@x"
        );
        assert_eq!(
            urlencoding::decode(parsed.password().unwrap()).unwrap(),
            "p@ss/w:rd#1?"
        );
    }

    // The fallback connects with the pool and the timeouts of the `.env`, and
    // `pool` sizes both.
    #[test]
    fn the_fallback_shares_the_pool_settings() {
        let mut vars = with_url("postgres://app:secret@db.local/shop", &SERVER_KEYS);
        vars.extend([("DB_MAX_CONNECTIONS", "7"), ("DB_CONNECT_TIMEOUT", "9")]);
        let mut config = rule(&vars).unwrap();
        let keys = config.fallback.clone().unwrap().unwrap();
        assert_eq!((config.max_connections, keys.max_connections), (7, 7));
        assert_eq!(keys.connect_timeout, Duration::from_secs(9));

        config.pool(1, 1);
        let keys = config.fallback.clone().unwrap().unwrap();
        assert_eq!((config.max_connections, config.min_connections), (1, 1));
        assert_eq!((keys.max_connections, keys.min_connections), (1, 1));
    }

    /// The keys of a SQLite file under `dir`, and the address they build.
    #[cfg(feature = "sqlite")]
    fn sqlite_keys(dir: &crate::atomic::tests::TempDir) -> (String, String) {
        let file = dir.join("base.sqlite").to_str().unwrap().to_string();
        let url = format!("sqlite://{file}?mode=rwc");
        (file, url)
    }

    /// Nothing listens on port 1.
    #[cfg(feature = "sqlite")]
    const UNREACHABLE: &str = "postgres://app:s3cr3t-Pw@127.0.0.1:1/shop";

    // Case A: the address doesn't answer, the keys do. The configuration that
    // answered is the keys' one.
    #[cfg(feature = "sqlite")]
    #[tokio::test]
    async fn an_address_that_fails_falls_back_on_the_keys() {
        let dir = crate::atomic::tests::TempDir::new();
        let (file, keys_url) = sqlite_keys(&dir);
        let config = rule(&with_url(
            UNREACHABLE,
            &[("DB_ENGINE", "sqlite"), ("DB_NAME", &file)],
        ))
        .unwrap();

        let (_conn, answered) = config.connect_resolved().await.unwrap();
        assert_eq!(answered.url, keys_url);
        assert!(config.connect().await.is_ok());
    }

    // Case A, keys that are not enough: blocked, with the first failure and
    // what is missing, and never the password.
    #[cfg(feature = "sqlite")]
    #[tokio::test]
    async fn an_address_that_fails_with_partial_keys_says_both() {
        let config = rule(&with_url(UNREACHABLE, &SERVER_KEYS[..2])).unwrap();

        let error = config.connect().await.unwrap_err().to_string();
        assert!(error.contains("DB_PASSWORD"), "{error}");
        assert!(!error.contains("s3cr3t-Pw"), "{error}");
    }

    // Case A, no key: the failure of the address, as before.
    #[cfg(feature = "sqlite")]
    #[tokio::test]
    async fn an_address_that_fails_alone_is_the_error() {
        let config = rule(&with_url(UNREACHABLE, &[])).unwrap();
        let error = config.connect().await.unwrap_err().to_string();
        assert!(!error.contains("s3cr3t-Pw"), "{error}");
    }

    // An address that answers is the one that answered, keys or not.
    #[cfg(feature = "sqlite")]
    #[tokio::test]
    async fn an_address_that_answers_is_kept() {
        let dir = crate::atomic::tests::TempDir::new();
        let (_, url) = sqlite_keys(&dir);
        let config = rule(&with_url(&url, &SERVER_KEYS)).unwrap();

        let (_conn, answered) = config.connect_resolved().await.unwrap();
        assert_eq!(answered.url, url);
    }

    // Line 2, connected: the keys answer in place of an incomplete address.
    #[cfg(feature = "sqlite")]
    #[tokio::test]
    async fn keys_answer_for_an_incomplete_address() {
        let dir = crate::atomic::tests::TempDir::new();
        let (file, keys_url) = sqlite_keys(&dir);
        let config = rule(&with_url(
            "mariadb://db.local/shop",
            &[("DB_ENGINE", "sqlite"), ("DB_NAME", &file)],
        ))
        .unwrap();

        let (_conn, answered) = config.connect_resolved().await.unwrap();
        assert_eq!(answered.url, keys_url);
    }

    #[test]
    fn test_serialize_skips_url() {
        let config = DatabaseConfig::from_url("postgres://myuser:secret123@localhost:5432/mydb")
            .unwrap()
            .build();
        let json = serde_json::to_value(&config).unwrap();
        assert!(json.get("url").is_none());
        assert!(!json.to_string().contains("secret123"));
    }

    #[test]
    fn test_detect_engine() {
        assert_eq!(
            DatabaseEngine::detect_from_url("postgres://localhost/db").unwrap(),
            DatabaseEngine::PostgreSQL
        );
        assert_eq!(
            DatabaseEngine::detect_from_url("mysql://localhost/db").unwrap(),
            DatabaseEngine::MySQL
        );
        assert_eq!(
            DatabaseEngine::detect_from_url("sqlite://db.sqlite").unwrap(),
            DatabaseEngine::SQLite
        );
    }
}
