//! `DatabaseConfigBuilder` — fluent API for configuring database parameters.
use std::time::Duration;

use super::config::DatabaseConfig;

/// Builder for `DatabaseConfig`.
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
///     .connect_timeout(Duration::from_secs(2))
///     .logging(true)
///     .build();
/// # Ok(())
/// # }
/// ```
pub struct DatabaseConfigBuilder {
    pub(super) config: DatabaseConfig,
}

impl DatabaseConfigBuilder {
    /// Sets the maximum number of connections in the pool.
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

    /// Sets the minimum number of connections in the pool.
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

    /// Sets the connection timeout.
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

    /// Sets both minimum and maximum pool size.
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

    /// Enables or disables SQL query logging.
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

    /// Builds the final `DatabaseConfig`.
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
