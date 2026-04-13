/// Moteur de base de données détecté au moment de la compilation (depuis `.env`).
/// Utilisé par la proc-macro pour adapter la génération selon l'engine cible.
#[derive(Debug, Clone, PartialEq)]
pub enum DbEngine {
    Postgres,
    Mysql,
    Sqlite,
    Unknown,
}

impl DbEngine {
    /// Détecte le moteur depuis l'environnement de build.
    ///
    /// Priorité :
    /// 1. Variable `DB_ENGINE` (override explicite : `postgres`, `mysql`, `sqlite`)
    /// 2. Préfixe de `DATABASE_URL` (`postgres://`, `mysql://`, `sqlite:`)
    /// 3. Recherche `.env` dans CWD, puis dans les parents jusqu'à 4 niveaux
    /// 4. Fallback : `Unknown`
    pub fn detect() -> Self {
        // 1. CARGO_MANIFEST_DIR — chemin de la crate compilée (le plus fiable en proc-macro).
        if let Ok(manifest_dir) = std::env::var("CARGO_MANIFEST_DIR") {
            let candidate = std::path::Path::new(&manifest_dir).join(".env");
            if candidate.exists() {
                let _ = dotenvy::from_path(&candidate);
            }
        }

        // 2. CWD puis remontée arborescence (fallback workspace racine).
        if std::env::var("DATABASE_URL").is_err() && std::env::var("DB_ENGINE").is_err() {
            let _ = dotenvy::dotenv();
        }
        if std::env::var("DATABASE_URL").is_err() && std::env::var("DB_ENGINE").is_err() {
            let mut dir = std::env::current_dir().ok();
            for _ in 0..4 {
                if let Some(d) = dir {
                    let candidate = d.join(".env");
                    if candidate.exists() {
                        let _ = dotenvy::from_path(&candidate);
                        break;
                    }
                    dir = d.parent().map(|p| p.to_path_buf());
                } else {
                    break;
                }
            }
        }

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

        DbEngine::Unknown
    }

    pub fn is_postgres(&self) -> bool {
        matches!(self, DbEngine::Postgres)
    }

    pub fn is_unknown(&self) -> bool {
        matches!(self, DbEngine::Unknown)
    }
}
