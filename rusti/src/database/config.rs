use sea_orm::{ConnectOptions, DatabaseConnection, Database, DbErr};
use std::time::Duration;
use dotenvy::dotenv;
use std::env;

/// Configuration avancée de la base de données
#[derive(Debug, Clone)]
pub struct DatabaseConfig {
    pub url: String,
    pub engine: DatabaseEngine,
    pub max_connections: u32,
    pub min_connections: u32,
    pub connect_timeout: Duration,
    pub acquire_timeout: Duration,
    pub idle_timeout: Duration,
    pub max_lifetime: Duration,
    pub sqlx_logging: bool,
}

/// Bases de données supportées par SeaORM
#[derive(Debug, Clone, PartialEq)]
pub enum DatabaseEngine {
    PostgreSQL,
    MySQL,
    MariaDB,
    SQLite,
}

impl DatabaseEngine {
    /// Détecte la BDD depuis une URL
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

    /// Nom de la base de données
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
    /// Builder avec détection automatique de la BDD depuis l'URL
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
            }
        })
    }

    /// Détecte depuis les variables d'environnement
    pub fn from_env() -> Result<DatabaseConfigBuilder, String> {
        dotenv().ok();

        let engine = env::var("DB_ENGINE")
            .unwrap_or_else(|_| "sqlite".to_string());

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

    /// Connecte à la base de données
    pub async fn connect(&self) -> Result<DatabaseConnection, DbErr> {
        tracing::info!("🔌 Connecting to {} database...", self.engine.name());

        // Vérification que le driver est activé
        verify_database_driver(&self.engine)
            .map_err(|e| DbErr::Custom(e))?;

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
            .sqlx_logging_level(tracing::log::LevelFilter::Info);

        match Database::connect(opt).await {
            Ok(conn) => {
                tracing::info!("Database connected successfully ({})", self.engine.name());
                Ok(conn)
            }
            Err(e) => {
                tracing::error!("❌ Database connection failed");
                tracing::error!("Engine: {}", self.engine.name());
                tracing::error!("URL: {}", mask_password(&self.url));
                Err(e)
            }
        }
    }
}

/// Vérifie que le driver de base de données est disponible
fn verify_database_driver(engine: &DatabaseEngine) -> Result<(), String> {
    match engine {
        DatabaseEngine::PostgreSQL => {
            #[cfg(not(feature = "postgres"))]
            return Err(
                "❌ PostgreSQL driver not enabled.\n\n\
                To fix this, add the 'postgres' feature to rusti in your Cargo.toml:\n\n\
                [dependencies]\n\
                rusti = { version = \"0.1\", features = [\"postgres\"] }\n\n\
                Or enable all databases:\n\
                rusti = { version = \"0.1\", features = [\"all-databases\"] }".to_string()
            );

            #[cfg(feature = "postgres")]
            Ok(())  
        }
        DatabaseEngine::MySQL => {
            #[cfg(not(feature = "mysql"))]
            return Err(
                "❌ MySQL driver not enabled.\n\n\
                To fix this, add the 'mysql' feature to rusti in your Cargo.toml:\n\n\
                [dependencies]\n\
                rusti = { version = \"0.1\", features = [\"mysql\"] }\n\n\
                Or enable all databases:\n\
                rusti = { version = \"0.1\", features = [\"all-databases\"] }".to_string()
            );

            #[cfg(feature = "mysql")]
            Ok(())
        }
        DatabaseEngine::MariaDB => {
            #[cfg(not(feature = "mariadb"))]
            return Err(
                "❌ MariaDB driver not enabled.\n\n\
                To fix this, add the 'mariadb' feature to rusti in your Cargo.toml:\n\n\
                [dependencies]\n\
                rusti = { version = \"0.1\", features = [\"mariadb\"] }\n\n\
                Note: MariaDB uses the MySQL driver.\n\n\
                Or enable all databases:\n\
                rusti = { version = \"0.1\", features = [\"all-databases\"] }".to_string()
            );

            #[cfg(feature = "mariadb")]
            Ok(())
        }
        DatabaseEngine::SQLite => {
            #[cfg(not(feature = "sqlite"))]
            return Err(
                "❌ SQLite driver not enabled.\n\n\
                To fix this, add the 'sqlite' feature to rusti in your Cargo.toml:\n\n\
                [dependencies]\n\
                rusti = { version = \"0.1\", features = [\"sqlite\"] }\n\n\
                Or use the default features (SQLite is enabled by default):\n\
                rusti = \"0.1\"".to_string()
            );

            #[cfg(feature = "sqlite")]
            Ok(())
        }
    }
}

/// Builder pour DatabaseConfig
pub struct DatabaseConfigBuilder {
    config: DatabaseConfig,
}

impl DatabaseConfigBuilder {
    pub fn max_connections(mut self, max: u32) -> Self {
        self.config.max_connections = max;
        self
    }

    pub fn min_connections(mut self, min: u32) -> Self {
        self.config.min_connections = min;
        self
    }

    pub fn connect_timeout(mut self, timeout: Duration) -> Self {
        self.config.connect_timeout = timeout;
        self
    }

    pub fn pool_size(mut self, min: u32, max: u32) -> Self {
        self.config.min_connections = min;
        self.config.max_connections = max;
        self
    }

    pub fn logging(mut self, enabled: bool) -> Self {
        self.config.sqlx_logging = enabled;
        self
    }

    pub fn build(self) -> DatabaseConfig {
        self.config
    }
}

/// Masque le mot de passe dans l'URL pour les logs
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