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

/// Base de données supportés par SeaORM
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
            "postgres" | "postgresql" => {
                let user = env::var("DB_USER")
                    .map_err(|_| "DB_USER not set for PostgreSQL")?;
                let password = env::var("DB_PASSWORD")
                    .map_err(|_| "DB_PASSWORD not set for PostgreSQL")?;
                let host = env::var("DB_HOST")
                    .unwrap_or_else(|_| "localhost".to_string());
                let port = env::var("DB_PORT")
                    .unwrap_or_else(|_| "5432".to_string());
                let name = env::var("DB_NAME")
                    .map_err(|_| "DB_NAME not set for PostgreSQL")?;

                format!("postgres://{}:{}@{}:{}/{}", user, password, host, port, name)
            }
            "mysql" => {
                let user = env::var("DB_USER")
                    .map_err(|_| "DB_USER not set for MySQL")?;
                let password = env::var("DB_PASSWORD")
                    .map_err(|_| "DB_PASSWORD not set for MySQL")?;
                let host = env::var("DB_HOST")
                    .unwrap_or_else(|_| "localhost".to_string());
                let port = env::var("DB_PORT")
                    .unwrap_or_else(|_| "3306".to_string());
                let name = env::var("DB_NAME")
                    .map_err(|_| "DB_NAME not set for MySQL")?;

                format!("mysql://{}:{}@{}:{}/{}", user, password, host, port, name)
            }
            "mariadb" => {
                let user = env::var("DB_USER")
                    .map_err(|_| "DB_USER not set for MariaDB")?;
                let password = env::var("DB_PASSWORD")
                    .map_err(|_| "DB_PASSWORD not set for MariaDB")?;
                let host = env::var("DB_HOST")
                    .unwrap_or_else(|_| "localhost".to_string());
                let port = env::var("DB_PORT")
                    .unwrap_or_else(|_| "3306".to_string());
                let name = env::var("DB_NAME")
                    .map_err(|_| "DB_NAME not set for MariaDB")?;

                format!("mariadb://{}:{}@{}:{}/{}", user, password, host, port, name)
            }
            "sqlite" => {
                let name = env::var("DB_NAME")
                    .unwrap_or_else(|_| "local_base.sqlite".to_string());
                format!("sqlite://{}?mode=rwc", name)
            }
            other => {
                env::var("DB_URL")
                    .map_err(|_| format!("Unsupported DB_ENGINE: {}", other))?
            }
        };

        Self::from_url(url)
    }

    /// Connecte à la base de données
    pub async fn connect(&self) -> Result<DatabaseConnection, DbErr> {
        tracing::info!("🔌 Connecting to {} database...", self.engine.name());

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
                tracing::info!("✅ Database connected successfully ({})", self.engine.name());
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