//! Database engine detection and driver verification.
use serde::{Deserialize, Serialize};

/// Database engines supported by Runique.
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
    /// Automatically detects the database type from a connection URL.
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
    /// Returns an error if the URL doesn't match any supported database.
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

    /// Returns the human-readable name of the database.
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

/// Verifies that the database driver is available.
///
/// # Errors
///
/// Returns a helpful error message if the corresponding Cargo feature is not enabled.
pub(super) fn verify_database_driver(engine: &DatabaseEngine) -> Result<(), String> {
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
