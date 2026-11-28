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
        } else if url.starts_with("sqlite:") {
            // No `//` required: `sqlite::memory:` is the only in-memory form
            // SeaORM accepts.
            Ok(DatabaseEngine::SQLite)
        } else {
            Err(
                runique_i18n::t(runique_i18n::keys::database::error::UNSUPPORTED_URL_SCHEME)
                    .into_owned(),
            )
        }
    }

    /// The engine a `DB_ENGINE` keyword names, whatever its case and the
    /// spaces around it: `postgres` or `postgresql`, `mysql`, `sqlite`.
    /// `None` for anything else.
    ///
    /// # Examples
    ///
    /// ```
    /// use runique::prelude::DatabaseEngine;
    ///
    /// assert_eq!(DatabaseEngine::from_keyword(" Postgres "), Some(DatabaseEngine::PostgreSQL));
    /// assert_eq!(DatabaseEngine::from_keyword("oracle"), None);
    /// ```
    pub fn from_keyword(keyword: &str) -> Option<Self> {
        match keyword.trim().to_ascii_lowercase().as_str() {
            "postgres" | "postgresql" => Some(DatabaseEngine::PostgreSQL),
            "mysql" => Some(DatabaseEngine::MySQL),
            "sqlite" => Some(DatabaseEngine::SQLite),
            _ => None,
        }
    }

    /// The engine a project's variables name, by the one rule every part of
    /// runique follows: `DATABASE_URL` decides when its start names an
    /// engine, `DB_ENGINE` otherwise (address absent, blank, or with a start
    /// that names none). `None` when neither names an engine.
    pub fn from_lookup(get: impl Fn(&str) -> Option<String>) -> Option<Self> {
        get("DATABASE_URL")
            .and_then(|url| Self::detect_from_url(&url).ok())
            .or_else(|| Self::from_keyword(&get("DB_ENGINE")?))
    }

    /// What a connection URL to this engine starts with.
    pub(crate) fn scheme(&self) -> &'static str {
        match self {
            DatabaseEngine::PostgreSQL => "postgres",
            DatabaseEngine::MySQL => "mysql",
            DatabaseEngine::SQLite => "sqlite",
        }
    }

    /// The port the engine's server listens on unless told otherwise. `None`
    /// for SQLite: a file, with no server and no account.
    pub(crate) fn default_port(&self) -> Option<&'static str> {
        match self {
            DatabaseEngine::PostgreSQL => Some("5432"),
            DatabaseEngine::MySQL => Some("3306"),
            DatabaseEngine::SQLite => None,
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
    runique_i18n::tf(
        runique_i18n::keys::build::error::DRIVER_NOT_ENABLED,
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
            msg.contains("cargo install runique-cli --features postgres --locked --force"),
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

    #[test]
    fn a_keyword_names_its_engine_whatever_its_case() {
        use super::DatabaseEngine;
        for (keyword, engine) in [
            ("sqlite", DatabaseEngine::SQLite),
            ("postgres", DatabaseEngine::PostgreSQL),
            ("postgresql", DatabaseEngine::PostgreSQL),
            ("mysql", DatabaseEngine::MySQL),
            ("  PostgreSQL\t", DatabaseEngine::PostgreSQL),
            ("SQLite", DatabaseEngine::SQLite),
        ] {
            assert_eq!(
                DatabaseEngine::from_keyword(keyword),
                Some(engine),
                "{keyword:?}"
            );
        }
        for unknown in [
            "",
            "oracle",
            "mariadb",
            "postgres://localhost/app",
            "my sql",
        ] {
            assert_eq!(DatabaseEngine::from_keyword(unknown), None, "{unknown:?}");
        }
    }

    #[test]
    fn the_database_url_decides_before_the_keyword() {
        use super::DatabaseEngine;
        let engine_of = |vars: &[(&str, &str)]| {
            DatabaseEngine::from_lookup(|key| {
                vars.iter()
                    .find(|(name, _)| *name == key)
                    .map(|(_, value)| value.to_string())
            })
        };

        assert_eq!(engine_of(&[]), None);
        assert_eq!(
            engine_of(&[("DB_ENGINE", "mysql")]),
            Some(DatabaseEngine::MySQL)
        );
        assert_eq!(
            engine_of(&[("DATABASE_URL", "sqlite://data.db")]),
            Some(DatabaseEngine::SQLite)
        );
        assert_eq!(
            engine_of(&[
                ("DB_ENGINE", "sqlite"),
                ("DATABASE_URL", "postgres://localhost/app"),
            ]),
            Some(DatabaseEngine::PostgreSQL)
        );
        // An address whose start names no engine leaves the choice to the key.
        assert_eq!(
            engine_of(&[
                ("DB_ENGINE", "postgres"),
                ("DATABASE_URL", "oracle://localhost/app"),
            ]),
            Some(DatabaseEngine::PostgreSQL)
        );
        assert_eq!(
            engine_of(&[("DB_ENGINE", "mysql"), ("DATABASE_URL", "")]),
            Some(DatabaseEngine::MySQL)
        );
        assert_eq!(
            engine_of(&[("DATABASE_URL", "oracle://localhost/app")]),
            None
        );
    }

    // A URL made of an engine's scheme is read back as that engine: the two
    // can't drift apart.
    #[test]
    fn each_engine_reads_back_from_its_own_scheme() {
        use super::DatabaseEngine;
        for (engine, scheme, port) in [
            (DatabaseEngine::PostgreSQL, "postgres", Some("5432")),
            (DatabaseEngine::MySQL, "mysql", Some("3306")),
            (DatabaseEngine::SQLite, "sqlite", None),
        ] {
            assert_eq!(engine.scheme(), scheme);
            assert_eq!(engine.default_port(), port);
            assert_eq!(
                DatabaseEngine::detect_from_url(&format!("{scheme}://app")),
                Ok(engine)
            );
        }
    }

    #[test]
    fn sqlite_is_read_with_or_without_slashes() {
        use super::DatabaseEngine;
        for url in ["sqlite::memory:", "sqlite:app.db", "sqlite://app.db"] {
            assert_eq!(
                DatabaseEngine::detect_from_url(url),
                Ok(DatabaseEngine::SQLite),
                "{url}"
            );
        }
        for url in ["sqlite", "sqlitex://app.db", "file:app.db"] {
            assert!(DatabaseEngine::detect_from_url(url).is_err(), "{url}");
        }
    }
}
