//! Core application staging: DB connection and URL registry.
use crate::app::error_build::{BuildError, CheckError, CheckReport};
use crate::utils::aliases::{ARlockmap, new_registry};

#[cfg(feature = "orm")]
use crate::db::DatabaseConfig;
#[cfg(feature = "orm")]
use sea_orm::DatabaseConnection;

// ═══════════════════════════════════════════════════════════════
// CoreStaging — Mandatory core application components
// ═══════════════════════════════════════════════════════════════
//
// Inspired by the Prisme forms system:
// - Flexible collection (the dev adds in the order they want)
// - Strict validation (everything is verified before construction)
// - OK signal (is_ready)
//
// CoreStaging accepts two paths for the DB:
//   1. .with_database(db)         → connection already established by the dev
//   2. .with_database_config(cfg) → staging validates the driver and
//                                    connects during build
// ═══════════════════════════════════════════════════════════════

pub struct CoreStaging {
    /// Already established DB connection (path 1)
    #[cfg(feature = "orm")]
    pub(crate) db: Option<DatabaseConnection>,

    /// DB configuration for deferred connection (path 2)
    #[cfg(feature = "orm")]
    pub(crate) db_config: Option<DatabaseConfig>,

    pub(crate) url_registry: ARlockmap,
}

impl CoreStaging {
    /// Creates a new CoreStaging with default values
    pub fn new() -> Self {
        Self {
            #[cfg(feature = "orm")]
            db: None,
            #[cfg(feature = "orm")]
            db_config: None,
            url_registry: new_registry(),
        }
    }

    // ═══════════════════════════════════════════════════
    // Database configuration
    // ═══════════════════════════════════════════════════

    /// Registers an already established database connection
    ///
    /// The dev manages the connection themselves:
    /// ```rust,ignore
    /// .core(|c| {
    ///     let db = DatabaseConfig::from_env()?.build().connect().await?;
    ///     c.with_database(db)
    /// })
    /// ```
    #[cfg(feature = "orm")]
    pub fn with_database(mut self, db: DatabaseConnection) -> Self {
        self.db = Some(db);
        self
    }

    /// Registers a DB configuration — the connection will be established during build.
    ///
    /// Staging validates the driver and connects automatically:
    /// ```rust,ignore
    /// .core(|c| {
    ///     let config = DatabaseConfig::from_env()?.build();
    ///     c.with_database_config(config)
    /// })
    /// ```
    #[cfg(feature = "orm")]
    pub fn with_database_config(mut self, config: DatabaseConfig) -> Self {
        self.db_config = Some(config);
        self
    }

    /// Validates that all mandatory components are present.
    ///
    /// Returns a `BuildError::CheckFailed` with a detailed report
    /// including corrective suggestions for each missing component.
    pub fn validate(&self) -> Result<(), BuildError> {
        let mut report = CheckReport::new();

        #[cfg(feature = "orm")]
        if self.db.is_none() && self.db_config.is_none() {
            report.add(
                CheckError::new(
                    "Database",
                    "Database connection or configuration required (`orm` feature enabled)",
                )
                .with_suggestion(
                    "Add .with_database(db) or .with_database_config(config) to your construction chain",
                ),
            );
        }

        if report.has_errors() {
            return Err(BuildError::check(report));
        }

        Ok(())
    }

    /// If a `DatabaseConfig` was provided (path 2), validates the driver
    /// and establishes the connection. If a direct connection was provided
    /// (path 1), returns it as is.
    ///
    /// Called during `build()` — after `validate()`.
    #[cfg(feature = "orm")]
    pub(crate) async fn connect(&mut self) -> Result<DatabaseConnection, BuildError> {
        // Path 1: connection already provided by the dev
        if let Some(db) = self.db.take() {
            return Ok(db);
        }

        // Path 2: connection from DatabaseConfig
        if let Some(config) = self.db_config.take() {
            let db = config.connect().await.map_err(|e| {
                BuildError::validation(format!("Failed to connect to the database: {}", e))
            })?;
            return Ok(db);
        }

        // Should never happen if validate() was called before
        Err(BuildError::database_missing())
    }

    /// Checks if the core is ready for construction
    pub fn is_ready(&self) -> bool {
        #[cfg(feature = "orm")]
        {
            if self.db.is_none() && self.db_config.is_none() {
                return false;
            }
        }
        true
    }
}

impl Default for CoreStaging {
    fn default() -> Self {
        Self::new()
    }
}
