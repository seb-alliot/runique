/// Moteur de base de données détecté au moment de la compilation (depuis `.env`).
/// Utilisé par la proc-macro pour adapter la génération selon l'engine cible.
#[derive(Debug, Clone, PartialEq)]
pub enum DbEngine {
    Postgres,
    Mysql,
    Sqlite,
}

impl DbEngine {
    /// Détecte le moteur depuis l'environnement de build.
    ///
    /// Priorité :
    /// 1. Variable `DB_ENGINE` (override explicite : `postgres`, `mysql`, `sqlite`)
    /// 2. Préfixe de `DATABASE_URL` (`postgres://`, `mysql://`, `sqlite:`)
    /// 3. Fallback : `Sqlite`
    ///
    /// Charge `.env` automatiquement avant de lire les variables.
    pub fn detect() -> Self {
        // Charge .env sans planter si absent (variables déjà dans l'env du build)
        let _ = dotenvy::dotenv();

        // 1. Override explicite DB_ENGINE
        if let Ok(engine) = std::env::var("DB_ENGINE") {
            match engine.to_ascii_lowercase().as_str() {
                "postgres" | "postgresql" => return DbEngine::Postgres,
                "mysql" | "mariadb" => return DbEngine::Mysql,
                "sqlite" => return DbEngine::Sqlite,
                _ => {}
            }
        }

        // 2. Préfixe DATABASE_URL
        if let Ok(url) = std::env::var("DATABASE_URL") {
            if url.starts_with("postgres://") || url.starts_with("postgresql://") {
                return DbEngine::Postgres;
            }
            if url.starts_with("mysql://") || url.starts_with("mariadb://") {
                return DbEngine::Mysql;
            }
            if url.starts_with("sqlite:") || url.starts_with("sqlite://") {
                return DbEngine::Sqlite;
            }
        }

        // 3. Fallback
        DbEngine::Sqlite
    }

    pub fn is_postgres(&self) -> bool {
        matches!(self, DbEngine::Postgres)
    }
}
