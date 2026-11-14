//! The project's `migration` crate: written by `runique new`, or by the first
//! `makemigrations` of a project that has none. Runique writes it itself rather
//! than `sea-orm-cli migrate init`, whose template pins neither the SeaORM
//! version runique is built with nor any database driver, and lacks runique.
use crate::atomic::{AtomicBuilder, SourceItems};
use crate::utils::trad::{t, tf};
use anyhow::Result;
use std::path::{Path, PathBuf};

/// The exact SeaORM requirement runique is built with, read from its manifest
/// by `build.rs`.
pub(crate) const SEA_ORM_REQ: &str = env!("RUNIQUE_SEA_ORM_REQ");
pub(crate) const SEA_ORM_MIGRATION_REQ: &str = env!("RUNIQUE_SEA_ORM_MIGRATION_REQ");

/// A database engine as a runique feature and the sqlx driver of
/// `sea-orm-migration` that goes with it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Engine {
    Sqlite,
    Postgres,
    Mysql,
    Mariadb,
}

impl Engine {
    /// From `DB_ENGINE`, as the app reads it; `None` when unset or unknown.
    pub(crate) fn from_env() -> Option<Self> {
        match std::env::var("DB_ENGINE")
            .ok()?
            .trim()
            .to_ascii_lowercase()
            .as_str()
        {
            "sqlite" => Some(Self::Sqlite),
            "postgres" | "postgresql" => Some(Self::Postgres),
            "mysql" => Some(Self::Mysql),
            "mariadb" => Some(Self::Mariadb),
            _ => None,
        }
    }

    fn feature(self) -> &'static str {
        match self {
            Self::Sqlite => "sqlite",
            Self::Postgres => "postgres",
            Self::Mysql => "mysql",
            Self::Mariadb => "mariadb",
        }
    }

    fn sqlx_feature(self) -> &'static str {
        match self {
            Self::Sqlite => "sqlx-sqlite",
            Self::Postgres => "sqlx-postgres",
            Self::Mysql | Self::Mariadb => "sqlx-mysql",
        }
    }
}

/// `Cargo.toml`, `src/main.rs` and `src/lib.rs` of the crate under
/// `migration_dir`, as (path, content). The `lib.rs` holds the framework's
/// migrations only, exactly what `makemigrations` would write for a project
/// without a model of its own.
pub(crate) fn files(
    migration_dir: &Path,
    engine: Engine,
    runique_version: &str,
) -> Vec<(PathBuf, Vec<u8>)> {
    let cargo = include_str!("composant-bin/migration_cargo")
        .replace("{{RUNIQUE_VERSION}}", runique_version)
        .replace("{{DB_FEATURE}}", engine.feature())
        .replace("{{SEA_ORM_MIGRATION_REQ}}", SEA_ORM_MIGRATION_REQ)
        .replace("{{SQLX_FEATURE}}", engine.sqlx_feature());
    vec![
        (migration_dir.join("Cargo.toml"), cargo.into_bytes()),
        (
            migration_dir.join("src/main.rs"),
            include_bytes!("composant-bin/migration_main.rs").to_vec(),
        ),
        (
            migration_dir.join("src/lib.rs"),
            crate::migration::cli_migration::initial_lib().into_bytes(),
        ),
    ]
}

/// Writes the migration crate as one batch: all three files, or none.
pub(crate) fn write(migration_dir: &Path, engine: Engine, runique_version: &str) -> Result<()> {
    let mut batch = AtomicBuilder::create();
    for (path, content) in files(migration_dir, engine, runique_version) {
        batch = batch.add(path, SourceItems::Bytes(content), false)?;
    }
    batch.start()?;
    Ok(())
}

/// For `makemigrations`: writes the crate when the project has none, with the
/// engine of `DB_ENGINE` (SQLite, said so, when unset). Only for the usual
/// `<crate>/src` layout, and only when the crate's folder doesn't exist at
/// all: an existing one, even without `Cargo.toml`, may hold migrations that
/// must not be overwritten.
pub(crate) fn ensure(migrations_path: &str) -> Result<()> {
    let path = Path::new(migrations_path);
    let Some(migration_dir) = path
        .file_name()
        .filter(|name| *name == "src")
        .and(path.parent())
        .filter(|dir| !dir.as_os_str().is_empty())
    else {
        return Ok(());
    };
    if migration_dir.exists() {
        return Ok(());
    }
    let engine = Engine::from_env().unwrap_or_else(|| {
        println!(
            "{}",
            tf(
                "migrate.engine_defaulted",
                &[migration_dir.join("Cargo.toml").display()]
            )
        );
        Engine::Sqlite
    });
    write(migration_dir, engine, env!("CARGO_PKG_VERSION"))?;
    println!(
        "{}",
        tf("migrate.crate_created", &[migration_dir.display()])
    );
    println!("{}", t("makemigrations.apply_hint"));
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn the_requirements_come_from_the_manifest() {
        assert!(SEA_ORM_REQ.starts_with('='), "{SEA_ORM_REQ}");
        assert!(
            SEA_ORM_MIGRATION_REQ.starts_with('='),
            "{SEA_ORM_MIGRATION_REQ}"
        );
    }

    #[test]
    fn the_crate_pins_runique_s_versions_and_the_engine_s_driver() {
        let dir = std::env::temp_dir().join(format!("rq_mig_{}", uuid::Uuid::new_v4()));
        write(&dir, Engine::Postgres, "9.8.7").unwrap();
        let cargo = fs::read_to_string(dir.join("Cargo.toml")).unwrap();
        assert!(!cargo.contains("{{"), "{cargo}");
        assert!(cargo.contains(r#"runique = { version = "9.8.7", features = ["postgres"] }"#));
        assert!(cargo.contains(&format!(r#"version = "{SEA_ORM_MIGRATION_REQ}""#)));
        assert!(cargo.contains(r#""sqlx-postgres""#));
        let lib = fs::read_to_string(dir.join("src/lib.rs")).unwrap();
        assert!(
            lib.contains("migrations_table::EihwazUsersMigration"),
            "{lib}"
        );
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn ensure_writes_a_missing_crate_and_never_touches_an_existing_folder() {
        let root = std::env::temp_dir().join(format!("rq_mig_{}", uuid::Uuid::new_v4()));
        let src = root.join("migration/src");
        ensure(src.to_str().unwrap()).unwrap();
        assert!(root.join("migration/Cargo.toml").is_file());

        let kept = root.join("other/src");
        fs::create_dir_all(&kept).unwrap();
        fs::write(kept.join("lib.rs"), "// the user's").unwrap();
        ensure(kept.to_str().unwrap()).unwrap();
        assert_eq!(
            fs::read_to_string(kept.join("lib.rs")).unwrap(),
            "// the user's"
        );
        assert!(!root.join("other/Cargo.toml").exists());

        let flat = root.join("flat");
        ensure(flat.to_str().unwrap()).unwrap();
        assert!(
            !flat.exists(),
            "no crate guessed outside the <crate>/src layout"
        );
        let _ = fs::remove_dir_all(&root);
    }

    /// The docs' table of versions, and their Cargo.toml examples, follow the
    /// manifest: a SeaORM bump without them would tell users a version Cargo
    /// then refuses.
    #[test]
    fn the_docs_give_the_sea_orm_version_runique_is_built_with() {
        // One column in the docs for both crates.
        assert_eq!(SEA_ORM_REQ, SEA_ORM_MIGRATION_REQ);
        let docs = Path::new(env!("CARGO_MANIFEST_DIR")).join("../docs");
        let row = format!("| {} | `{SEA_ORM_REQ}` |", env!("CARGO_PKG_VERSION"));
        let snippet = format!(r#"sea-orm = "{SEA_ORM_REQ}""#);
        for (page, expected) in [
            ("fr/installation/base-de-donnees/base-de-donnees.md", &row),
            ("en/installation/database/database.md", &row),
            (
                "fr/installation/troubleshooting/troubleshooting.md",
                &snippet,
            ),
            (
                "en/installation/troubleshooting/troubleshooting.md",
                &snippet,
            ),
        ] {
            let text = fs::read_to_string(docs.join(page)).unwrap();
            assert!(
                text.contains(expected.as_str()),
                "{page}: `{expected}` missing"
            );
        }
    }

    #[test]
    fn mariadb_and_mysql_share_the_mysql_driver() {
        assert_eq!(Engine::Mariadb.sqlx_feature(), "sqlx-mysql");
        assert_eq!(Engine::Mysql.sqlx_feature(), "sqlx-mysql");
        assert_eq!(Engine::Mariadb.feature(), "mariadb");
    }
}
