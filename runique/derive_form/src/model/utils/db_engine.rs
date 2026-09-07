/// Database engine detected at compile time — via the Cargo feature forwarded
/// from `runique` in priority, `.env` as a fallback. Used by the proc-macro to
/// adapt generation according to the target engine.
#[derive(Debug, Clone, PartialEq)]
pub enum DbEngine {
    Postgres,
    Mysql,
    Sqlite,
    Unknown,
}

impl DbEngine {
    /// Detects the engine at compile time.
    ///
    /// Priority:
    /// 1. Feature Cargo forwardée depuis `runique` (`postgres`/`mysql`/`sqlite`) —
    ///    source de vérité : ne peut pas désynchroniser de ce qui est réellement
    ///    compilé, contrairement à `.env` qui peut mentir (mauvais fichier, valeur
    ///    oubliée après un changement de moteur).
    /// 2. Fallback `.env` (`DB_ENGINE` puis `DATABASE_URL`) — pour un projet dont le
    ///    `Cargo.toml` ne forwarde pas encore la feature vers `derive_form`.
    pub fn detect() -> Self {
        if cfg!(feature = "postgres") {
            return DbEngine::Postgres;
        }
        if cfg!(feature = "mysql") {
            return DbEngine::Mysql;
        }
        if cfg!(feature = "sqlite") {
            return DbEngine::Sqlite;
        }
        Self::detect_from_env()
    }

    /// Ancien mécanisme, conservé en repli tant que tous les projets n'ont pas
    /// migré vers le forwarding de features `runique` -> `derive_form`.
    fn detect_from_env() -> Self {
        // 1. CARGO_MANIFEST_DIR — path of the compiled crate (most reliable in proc-macro).
        if let Ok(manifest_dir) = std::env::var("CARGO_MANIFEST_DIR") {
            let candidate = std::path::Path::new(&manifest_dir).join(".env");
            if candidate.exists() {
                let _ = dotenvy::from_path(&candidate);
            }
        }

        // 2. CWD then traversing tree up (root workspace fallback).
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

        // 1. Explicit DB_ENGINE override
        if let Ok(engine) = std::env::var("DB_ENGINE") {
            match engine.to_ascii_lowercase().as_str() {
                "postgres" | "postgresql" => return DbEngine::Postgres,
                "mysql" | "mariadb" => return DbEngine::Mysql,
                "sqlite" => return DbEngine::Sqlite,
                _ => {}
            }
        }

        // 2. DATABASE_URL prefix
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
