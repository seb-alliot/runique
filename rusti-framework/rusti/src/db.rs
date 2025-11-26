use sea_orm::{ConnectOptions, Database, DatabaseConnection, DbErr};
use std::time::Duration;
use crate::settings::Settings;
use tracing::log;

/// Établit la connexion à la base de données
///
/// # Arguments
/// * `config` - Configuration de l'application contenant les paramètres DB
///
/// # Erreurs
/// Retourne une erreur si la connexion échoue ou si la configuration est invalide
///
/// # Exemple
/// ```rust,no_run
/// use rusti::{Settings, db::connect_db};
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let config = Settings::default_values();
///     let db = connect_db(&config).await?;
///     Ok(())
/// }
/// ```
pub async fn connect_db(config: &Settings) -> Result<DatabaseConnection, DbErr> {
    let db_config = &config.databases;

    // Validation pour les bases de données non-SQLite
    if db_config.engine != "sqlite" {
        validate_db_config(db_config)?;
    }

    // Configuration des options de connexion
    let mut opt = ConnectOptions::new(&db_config.url);
    opt.max_connections(20)
        .min_connections(5)
        .connect_timeout(Duration::from_secs(8))
        .acquire_timeout(Duration::from_secs(8))
        .idle_timeout(Duration::from_secs(300))
        .max_lifetime(Duration::from_secs(3600))
        .sqlx_logging(true)
        .sqlx_logging_level(log::LevelFilter::Info);

    // Tentative de connexion
    match Database::connect(opt).await {
        Ok(conn) => {
            tracing::info!("✅ Database connected successfully");
            Ok(conn)
        }
        Err(e) => {
            tracing::error!("❌ Database connection failed");
            tracing::error!("URL: {}", mask_password(&db_config.url));
            Err(e)
        }
    }
}

/// Valide la configuration de la base de données
fn validate_db_config(db_config: &crate::settings::DatabaseSettings) -> Result<(), DbErr> {
    let database_fields = [
        &db_config.user,
        &db_config.password,
        &db_config.host,
        &db_config.name
    ];

    for field in database_fields.iter() {
        if field.is_empty() {
            return Err(DbErr::Custom(format!(
                "Database configuration incomplete for '{}'. Check environment variables.",
                db_config.engine
            )));
        }
    }

    if db_config.port == 0 {
        return Err(DbErr::Custom(
            "Database port is missing or invalid.".to_string()
        ));
    }

    Ok(())
}

/// Masque le mot de passe dans l'URL pour les logs
///
/// Transforme `postgres://user:password@host/db` en `postgres://user:****@host/db`
fn mask_password(url: &str) -> String {
    if let Some(idx) = url.find("://") {
        if let Some(at_idx) = url[idx+3..].find('@') {
            let before = &url[..idx+3];
            let after = &url[idx+3+at_idx..];

            if let Some(colon) = url[idx+3..idx+3+at_idx].find(':') {
                let user = &url[idx+3..idx+3+colon];
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
}
