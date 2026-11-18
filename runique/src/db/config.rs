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
        // DATABASE_URL takes priority over component variables (compatible with sea-orm-cli)
        let url = if let Some(direct_url) = get("DATABASE_URL") {
            direct_url
        } else {
            let keyword = get("DB_ENGINE").unwrap_or_else(|| "sqlite".to_string());
            let engine = DatabaseEngine::from_keyword(&keyword).ok_or_else(|| {
                format!(
                    " Unsupported DB_ENGINE: {}\n\nSupported engines: postgres, mysql, sqlite\nOr set DATABASE_URL directly.",
                    keyword.trim().to_ascii_lowercase()
                )
            })?;

            match engine.default_port() {
                // A server: an account, a host and a port.
                Some(default_port) => {
                    let user = get("DB_USER")
                        .ok_or_else(|| format!(" DB_USER not set for {}\n\nRequired variables:\n  - DB_USER\n  - DB_PASSWORD\n  - DB_HOST (optional, default: localhost)\n  - DB_PORT (optional, default: {})\n  - DB_NAME", engine.name(), default_port))?;

                    let password = get("DB_PASSWORD")
                        .ok_or_else(|| format!(" DB_PASSWORD not set for {}", engine.name()))?;

                    let host = get("DB_HOST").unwrap_or_else(|| "localhost".to_string());
                    let port = get("DB_PORT").unwrap_or_else(|| default_port.to_string());
                    let name = get("DB_NAME")
                        .ok_or_else(|| format!(" DB_NAME not set for {}", engine.name()))?;

                    format!(
                        "{}://{}:{}@{}:{}/{}",
                        engine.scheme(),
                        user,
                        password,
                        host,
                        port,
                        name
                    )
                }
                // SQLite: a file.
                None => {
                    let name = get("DB_NAME").unwrap_or_else(|| "local_base.sqlite".to_string());
                    format!("{}://{}?mode=rwc", engine.scheme(), name)
                }
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
