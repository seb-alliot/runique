//! Database connection utilities (requires `orm` feature)

use sea_orm::{ConnectOptions, Database, DatabaseConnection, DbErr};
use std::time::Duration;
use crate::config::Settings;

/// Connect to the database using settings
pub async fn connect_db(config: &Settings) -> Result<DatabaseConnection, DbErr> {
    let db_config = &config.database;

    // Validate configuration for non-sqlite databases
    if db_config.engine != "sqlite" {
        validate_db_config(db_config)?;
    }

    // Setup connection options
    let mut opt = ConnectOptions::new(&db_config.url);
    opt.max_connections(20)
        .min_connections(5)
        .connect_timeout(Duration::from_secs(8))
        .acquire_timeout(Duration::from_secs(8))
        .idle_timeout(Duration::from_secs(300))
        .max_lifetime(Duration::from_secs(3600))
        .sqlx_logging(true)
        .sqlx_logging_level(log::LevelFilter::Info);

    // Connect
    match Database::connect(opt).await {
        Ok(conn) => {
            tracing::info!("Database connected successfully");
            Ok(conn)
        }
        Err(e) => {
            tracing::error!("Database connection failed: {}", e);
            tracing::error!("URL: {}", mask_password(&db_config.url));
            Err(e)
        }
    }
}

/// Validate database configuration
fn validate_db_config(config: &crate::config::DatabaseSettings) -> Result<(), DbErr> {
    let fields = [
        (&config.user, "USER"),
        (&config.password, "PASSWORD"),
        (&config.host, "HOST"),
        (&config.name, "NAME"),
    ];

    for (field, name) in fields.iter() {
        if field.is_empty() {
            return Err(DbErr::Custom(format!(
                "Database configuration incomplete: {} is missing",
                name
            )));
        }
    }

    if config.port == 0 {
        return Err(DbErr::Custom("Database port is invalid or missing".into()));
    }

    Ok(())
}

/// Mask password in database URL for logging
fn mask_password(url: &str) -> String {
    if let Some(idx) = url.find("://") {
        if let Some(at_idx) = url[idx + 3..].find('@') {
            let before = &url[..idx + 3];
            let after = &url[idx + 3 + at_idx..];

            if let Some(colon_idx) = url[idx + 3..idx + 3 + at_idx].find(':') {
                let user = &url[idx + 3..idx + 3 + colon_idx];
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
        let url = "postgres://user:secret@localhost:5432/db";
        let masked = mask_password(url);
        assert_eq!(masked, "postgres://user:****@localhost:5432/db");
    }

    #[test]
    fn test_mask_password_no_password() {
        let url = "sqlite://db.sqlite";
        let masked = mask_password(url);
        assert_eq!(masked, "sqlite://db.sqlite");
    }
}
