//! Database configuration and connection management
//!
//! This module provides flexible configuration for connecting to different
//! databases (PostgreSQL, MySQL, MariaDB, SQLite) via SeaORM.
//!
//! # Examples
//!
//! ```no_run
//! use runique::prelude::DatabaseConfig;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // From environment variables
//! let config = DatabaseConfig::from_env()?.build();
//! let db = config.connect().await?;
//!
//! // Or from a URL
//! let config = DatabaseConfig::from_url("sqlite://db.sqlite")?.build();
//! let db = config.connect().await?;
//! # Ok(())
//! # }
//! ```

use dotenvy::dotenv;
use sea_orm::{ConnectOptions, Database, DatabaseConnection, DbErr};
use serde::{Deserialize, Serialize};
use std::env;
use std::time::Duration;
/// Advanced database configuration
///
/// Contains all parameters needed to establish and manage a database
/// connection, including connection pools and timeouts.
///
/// # Examples
///
/// ```no_run
/// use runique::prelude::DatabaseConfig;
/// use std::time::Duration;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let config = DatabaseConfig::from_url("postgres://user:pass@localhost/db")?
///     .max_connections(50)
///     .connect_timeout(Duration::from_secs(10))
///     .build();
///
/// let db = config.connect().await?;
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    /// Database connection URL
    pub url: String,
    /// Database type (PostgreSQL, MySQL, MariaDB, SQLite)
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

/// Database engines supported by Runique
///
/// Each variant corresponds to a database driver that must be
/// enabled via Cargo features.
///
/// # Required Features
///
/// - `sqlite` - For SQLite (enabled by default)
/// - `postgres` - For PostgreSQL
/// - `mysql` - For MySQL
/// - `mariadb` - For MariaDB (uses MySQL driver)
///
/// # Examples
///
/// ```
/// use runique::prelude::DatabaseEngine;
///
/// let engine = DatabaseEngine::detect_from_url("postgres://localhost/db")?;
/// assert_eq!(engine, DatabaseEngine::PostgreSQL);
/// # Ok::<(), String>(())
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DatabaseEngine {
    /// PostgreSQL database
    PostgreSQL,
    /// MySQL database
    MySQL,
    /// MariaDB database (MySQL-compatible)
    MariaDB,
    /// SQLite embedded database
    SQLite,
}

impl DatabaseEngine {
    /// Automatically detects the database type from a connection URL
    ///
    /// # Arguments
    ///
    /// * `url` - Database connection URL
    ///
    /// # Examples
    ///
    /// ```
    /// use runique::prelude::DatabaseEngine;
    ///
    /// let engine = DatabaseEngine::detect_from_url("sqlite://db.sqlite")?;
    /// assert_eq!(engine, DatabaseEngine::SQLite);
    ///
    /// let engine = DatabaseEngine::detect_from_url("postgres://localhost/db")?;
    /// assert_eq!(engine, DatabaseEngine::PostgreSQL);
    /// # Ok::<(), String>(())
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if the URL doesn't match any supported database
    pub fn detect_from_url(url: &str) -> Result<Self, String> {
        if url.starts_with("postgres://") || url.starts_with("postgresql://") {
            Ok(DatabaseEngine::PostgreSQL)
        } else if url.starts_with("mysql://") {
            Ok(DatabaseEngine::MySQL)
        } else if url.starts_with("mariadb://") {
            Ok(DatabaseEngine::MariaDB)
        } else if url.starts_with("sqlite://") {
            Ok(DatabaseEngine::SQLite)
        } else {
            Err(format!("Unsupported database URL: {}", url))
        }
    }

    /// Returns the human-readable name of the database
    ///
    /// # Examples
    ///
    /// ```
    /// use runique::prelude::DatabaseEngine;
    ///
    /// assert_eq!(DatabaseEngine::PostgreSQL.name(), "PostgreSQL");
    /// assert_eq!(DatabaseEngine::SQLite.name(), "SQLite");
    /// ```
    pub fn name(&self) -> &'static str {
        match self {
            DatabaseEngine::PostgreSQL => "PostgreSQL",
            DatabaseEngine::MySQL => "MySQL",
            DatabaseEngine::MariaDB => "MariaDB",
            DatabaseEngine::SQLite => "SQLite",
        }
    }
}

impl DatabaseConfig {
    /// Creates a configuration from a connection URL
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
    /// Returns an error if the URL is invalid or unsupported
    pub fn from_url(url: impl Into<String>) -> Result<DatabaseConfigBuilder, String> {
        let url = url.into();
        let engine = DatabaseEngine::detect_from_url(&url)?;

        Ok(DatabaseConfigBuilder {
            config: DatabaseConfig {
                url,
                engine,
                max_connections: 20,
                min_connections: 5,
                connect_timeout: Duration::from_secs(8),
                acquire_timeout: Duration::from_secs(8),
                idle_timeout: Duration::from_secs(300),
                max_lifetime: Duration::from_secs(3600),
                sqlx_logging: true,
            },
        })
    }

    /// Creates a configuration from environment variables
    ///
    /// Reads configuration from a `.env` file or system environment variables.
    ///
    /// # Environment Variables
    ///
    /// - `DB_ENGINE` - Database type (postgres, mysql, mariadb, sqlite)
    /// - `DB_USER` - Username (required except for SQLite)
    /// - `DB_PASSWORD` - Password (required except for SQLite)
    /// - `DB_HOST` - Host (default: localhost)
    /// - `DB_PORT` - Port (default: 5432 for PostgreSQL, 3306 for MySQL)
    /// - `DB_NAME` - Database name
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use runique::prelude::DatabaseConfig;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// // .env file:
    /// // DB_ENGINE=postgres
    /// // DB_USER=myuser
    /// // DB_PASSWORD=secret
    /// // DB_NAME=mydb
    ///
    /// let config = DatabaseConfig::from_env()?.build();
    /// let db = config.connect().await?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if required variables are missing
    pub fn from_env() -> Result<DatabaseConfigBuilder, String> {
        dotenv().ok();

        let engine = env::var("DB_ENGINE").unwrap_or_else(|_| "sqlite".to_string());

        let url = match engine.as_str() {
            "postgres" | "postgresql" | "mysql" | "mariadb" => {
                let db_type = match engine.as_str() {
                    "postgres" | "postgresql" => ("postgres", "5432", "PostgreSQL"),
                    "mysql" => ("mysql", "3306", "MySQL"),
                    "mariadb" => ("mariadb", "3306", "MariaDB"),
                    _ => unreachable!(),
                };

                let user = env::var("DB_USER")
                    .map_err(|_| format!(" DB_USER not set for {}\n\nRequired variables:\n  - DB_USER\n  - DB_PASSWORD\n  - DB_HOST (optional, default: localhost)\n  - DB_PORT (optional, default: {})\n  - DB_NAME", db_type.2, db_type.1))?;

                let password = env::var("DB_PASSWORD")
                    .map_err(|_| format!(" DB_PASSWORD not set for {}", db_type.2))?;

                let host = env::var("DB_HOST")
                    .unwrap_or_else(|_| "localhost".to_string());

                let port = env::var("DB_PORT")
                    .unwrap_or_else(|_| db_type.1.to_string());

                let name = env::var("DB_NAME")
                    .map_err(|_| format!(" DB_NAME not set for {}", db_type.2))?;

                format!("{}://{}:{}@{}:{}/{}", db_type.0, user, password, host, port, name)
            }
            "sqlite" => {
                let name = env::var("DB_NAME")
                    .unwrap_or_else(|_| "local_base.sqlite".to_string());
                format!("sqlite://{}?mode=rwc", name)
            }
            other => {
                env::var("DB_URL")
                    .map_err(|_| format!(" Unsupported DB_ENGINE: {}\n\nSupported engines: postgres, mysql, mariadb, sqlite", other))?
            }
        };

        Self::from_url(url)
    }

    /// Establishes connection to the database
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
    ///
    /// // Use the connection...
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// - Returns an error if the driver is not enabled (missing Cargo feature)
    /// - Returns an error if the connection fails
    pub async fn connect(&self) -> Result<DatabaseConnection, DbErr> {
        tracing::info!("  Connecting to {} database...", self.engine.name());

        // Vérification que le driver est activé
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
                tracing::info!("Database connected successfully ({})", self.engine.name());
                Ok(conn)
            }
            Err(e) => {
                tracing::error!("└──> Database connection failed");
                tracing::error!("   └──> Engine: {}", self.engine.name());
                tracing::error!("       └──> URL: {}", mask_password(&self.url));
                Err(e)
            }
        }
    }
}

/// Verifies that the database driver is available
///
/// # Errors
///
/// Returns a helpful error message if the corresponding Cargo feature is not enabled
fn verify_database_driver(engine: &DatabaseEngine) -> Result<(), String> {
    match engine {
        DatabaseEngine::PostgreSQL => {
            #[cfg(not(feature = "postgres"))]
            return Err("PostgreSQL driver not enabled.\n\n\
                To fix this, add the 'postgres' feature to runique in your Cargo.toml:\n\n\
                [dependencies]\n\
                runique = { version = \"0.1\", features = [\"postgres\"] }\n\n\
                Or enable all databases:\n\
                runique = { version = \"0.1\", features = [\"all-databases\"] }"
                .to_string());

            #[cfg(feature = "postgres")]
            Ok(())
        }
        DatabaseEngine::MySQL => {
            #[cfg(not(feature = "mysql"))]
            return Err("MySQL driver not enabled.\n\n\
                To fix this, add the 'mysql' feature to runique in your Cargo.toml:\n\n\
                [dependencies]\n\
                runique = { version = \"0.1\", features = [\"mysql\"] }\n\n\
                Or enable all databases:\n\
                runique = { version = \"0.1\", features = [\"all-databases\"] }"
                .to_string());

            #[cfg(feature = "mysql")]
            Ok(())
        }
        DatabaseEngine::MariaDB => {
            #[cfg(not(feature = "mariadb"))]
            return Err("MariaDB driver not enabled.\n\n\
                To fix this, add the 'mariadb' feature to runique in your Cargo.toml:\n\n\
                [dependencies]\n\
                runique = { version = \"0.1\", features = [\"mariadb\"] }\n\n\
                Note: MariaDB uses the MySQL driver.\n\n\
                Or enable all databases:\n\
                runique = { version = \"0.1\", features = [\"all-databases\"] }"
                .to_string());

            #[cfg(feature = "mariadb")]
            Ok(())
        }
        DatabaseEngine::SQLite => {
            #[cfg(not(feature = "sqlite"))]
            return Err(
                "To fix this, add the 'sqlite' feature to runique in your Cargo.toml:\n\n\
                [dependencies]\n\
                runique = { version = \"1.xx\", features = [\"sqlite\"] }
                Note: Sqlite uses the Sqlite driver.\n\n\
                Or enable all databases:\n\
                runique = { version = \"0.1\", features = [\"all-databases\"]"
                    .to_string(),
            );

            #[cfg(feature = "sqlite")]
            Ok(())
        }
    }
}

/// Builder for `DatabaseConfig`
///
/// Allows fluent configuration of database parameters.
///
/// # Examples
///
/// ```no_run
/// use runique::prelude::DatabaseConfig;
/// use std::time::Duration;
///
/// # fn example() -> Result<(), String> {
/// let config = DatabaseConfig::from_url("postgres://localhost/db")?
///     .max_connections(50)
///     .min_connections(10)
///     .connect_timeout(Duration::from_secs(5))
///     .logging(true)
///     .build();
/// # Ok(())
/// # }
/// ```
pub struct DatabaseConfigBuilder {
    config: DatabaseConfig,
}

impl DatabaseConfigBuilder {
    /// Sets the maximum number of connections in the pool
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use runique::prelude::DatabaseConfig;
    ///
    /// let config = DatabaseConfig::from_url("postgres://localhost/db")?
    ///     .max_connections(100)
    ///     .build();
    /// # Ok::<(), String>(())
    /// ```
    pub fn max_connections(mut self, max: u32) -> Self {
        self.config.max_connections = max;
        self
    }

    /// Sets the minimum number of connections in the pool
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use runique::prelude::DatabaseConfig;
    ///
    /// let config = DatabaseConfig::from_url("postgres://localhost/db")?
    ///     .min_connections(5)
    ///     .build();
    /// # Ok::<(), String>(())
    /// ```
    pub fn min_connections(mut self, min: u32) -> Self {
        self.config.min_connections = min;
        self
    }

    /// Sets the connection timeout
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use runique::prelude::DatabaseConfig;
    /// use std::time::Duration;
    ///
    /// let config = DatabaseConfig::from_url("postgres://localhost/db")?
    ///     .connect_timeout(Duration::from_secs(10))
    ///     .build();
    /// # Ok::<(), String>(())
    /// ```
    pub fn connect_timeout(mut self, timeout: Duration) -> Self {
        self.config.connect_timeout = timeout;
        self
    }

    /// Sets both minimum and maximum pool size
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use runique::prelude::DatabaseConfig;
    ///
    /// let config = DatabaseConfig::from_url("postgres://localhost/db")?
    ///     .pool_size(10, 50)
    ///     .build();
    /// # Ok::<(), String>(())
    /// ```
    pub fn pool_size(mut self, min: u32, max: u32) -> Self {
        self.config.min_connections = min;
        self.config.max_connections = max;
        self
    }

    /// Enables or disables SQL query logging
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use runique::prelude::DatabaseConfig;
    ///
    /// let config = DatabaseConfig::from_url("postgres://localhost/db")?
    ///     .logging(false)
    ///     .build();
    /// # Ok::<(), String>(())
    /// ```
    pub fn logging(mut self, enabled: bool) -> Self {
        self.config.sqlx_logging = enabled;
        self
    }

    /// Builds the final `DatabaseConfig`
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use runique::prelude::DatabaseConfig;
    ///
    /// let config = DatabaseConfig::from_url("sqlite://db.sqlite")?
    ///     .build();
    /// # Ok::<(), String>(())
    /// ```
    pub fn build(self) -> DatabaseConfig {
        self.config
    }
}

/// Masks the password in a URL for logging purposes
fn mask_password(url: &str) -> String {
    if let Some(idx) = url.find("://") {
        if let Some(at_idx) = url[idx + 3..].find('@') {
            let before = &url[..idx + 3];
            let after = &url[idx + 3 + at_idx..];

            if let Some(colon) = url[idx + 3..idx + 3 + at_idx].find(':') {
                let user = &url[idx + 3..idx + 3 + colon];
                return format!("{}{}:****{}", before, user, after);
            }
        }
    }
    url.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

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
