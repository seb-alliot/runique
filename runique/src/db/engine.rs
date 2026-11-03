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
            Err("Unsupported database URL scheme".to_string())
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
        #[cfg(not(feature = "postgres"))]
        DatabaseEngine::PostgreSQL => Err(driver_not_enabled(engine, "postgres")),
        #[cfg(not(feature = "mysql"))]
        DatabaseEngine::MySQL => Err(driver_not_enabled(engine, "mysql")),
        // MariaDB goes through the MySQL driver: `all-databases` enables `mysql`, not `mariadb`.
        #[cfg(not(feature = "mysql"))]
        DatabaseEngine::MariaDB => Err(driver_not_enabled(engine, "mariadb")),
        #[cfg(not(feature = "sqlite"))]
        DatabaseEngine::SQLite => Err(driver_not_enabled(engine, "sqlite")),
        #[allow(unreachable_patterns)]
        _ => Ok(()),
    }
}

/// Says what runique was compiled with, then both ways out: point `.env` at
/// that engine (the usual mistake: the features changed, `.env` didn't), or
/// compile for the engine `.env` asks for (dependency line, `cargo install`).
#[cfg(not(all(feature = "postgres", feature = "mysql", feature = "sqlite")))]
fn driver_not_enabled(engine: &DatabaseEngine, feature: &str) -> String {
    let compiled = compiled_engines();
    crate::utils::trad::tf(
        "build.driver_not_enabled",
        &[
            engine.name(),
            compiled.as_str(),
            feature,
            compiled.as_str(),
            engine.name(),
            env!("CARGO_PKG_VERSION"),
            feature,
            feature,
        ],
    )
}

/// The engine features this build has, as their names.
#[cfg(not(all(feature = "postgres", feature = "mysql", feature = "sqlite")))]
fn compiled_engines() -> String {
    let engines: Vec<&str> = [
        (cfg!(feature = "postgres"), "postgres"),
        (cfg!(feature = "mysql"), "mysql"),
        (cfg!(feature = "sqlite"), "sqlite"),
    ]
    .into_iter()
    .filter_map(|(on, name)| on.then_some(name))
    .collect();
    if engines.is_empty() {
        "none".to_string()
    } else {
        engines.join(", ")
    }
}

#[cfg(test)]
mod tests {
    // Each test is compiled only for some features: the import lives in them.

    #[cfg(not(feature = "postgres"))]
    #[test]
    fn a_missing_driver_names_the_real_version_and_both_fixes() {
        use super::*;
        let msg = verify_database_driver(&DatabaseEngine::PostgreSQL).unwrap_err();
        assert!(msg.contains(env!("CARGO_PKG_VERSION")), "{msg}");
        assert!(msg.contains(r#"features = ["postgres"]"#), "{msg}");
        assert!(
            msg.contains("cargo install runique --features postgres --locked --force"),
            "{msg}"
        );
        // The other way out: `.env` pointing at the engine this build has.
        assert!(msg.contains(&format!("'{}'", compiled_engines())), "{msg}");
        assert!(
            msg.contains("DB_ENGINE") && msg.contains("DATABASE_URL"),
            "{msg}"
        );
    }

    #[cfg(feature = "sqlite")]
    #[test]
    fn a_compiled_driver_passes() {
        use super::*;
        assert!(verify_database_driver(&DatabaseEngine::SQLite).is_ok());
    }
}
