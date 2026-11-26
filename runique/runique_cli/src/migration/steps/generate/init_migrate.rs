//! The project's `migration` crate: written by `runique new`, or by the first
//! `makemigrations` of a project that has none. Runique writes it itself rather
//! than `sea-orm-cli migrate init`, whose template pins neither the SeaORM
//! version runique is built with nor any database driver, and lacks runique.
use crate::atomic::{AtomicBuilder, SourceItems};
use crate::db::engine::DatabaseEngine;
use anyhow::{Context, Result};
use runique_i18n::{t, tf};
use std::path::{Path, PathBuf};

/// The exact SeaORM requirement runique is built with, read from its manifest
/// by `build.rs`.
pub(crate) const SEA_ORM_REQ: &str = crate::tooling::SEA_ORM_REQ;
pub(crate) const SEA_ORM_MIGRATION_REQ: &str = crate::tooling::SEA_ORM_MIGRATION_REQ;

/// A database engine as a runique feature and the sqlx driver of
/// `sea-orm-migration` that goes with it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Engine {
    Sqlite,
    Postgres,
    Mysql,
}

impl From<DatabaseEngine> for Engine {
    fn from(engine: DatabaseEngine) -> Self {
        match engine {
            DatabaseEngine::SQLite => Self::Sqlite,
            DatabaseEngine::PostgreSQL => Self::Postgres,
            DatabaseEngine::MySQL => Self::Mysql,
        }
    }
}

impl Engine {
    /// The project's engine, read from its environment the way the app
    /// reads it.
    pub(crate) fn from_env() -> Option<Self> {
        Self::from_lookup(|key| std::env::var(key).ok())
    }

    /// The engine the app will connect to, so the crate is built for it:
    /// [`DatabaseEngine::from_lookup`], the app's own rule.
    fn from_lookup(get: impl Fn(&str) -> Option<String>) -> Option<Self> {
        DatabaseEngine::from_lookup(get).map(Self::from)
    }

    fn feature(self) -> &'static str {
        match self {
            Self::Sqlite => "sqlite",
            Self::Postgres => "postgres",
            Self::Mysql => "mysql",
        }
    }

    fn sqlx_feature(self) -> &'static str {
        match self {
            Self::Sqlite => "sqlx-sqlite",
            Self::Postgres => "sqlx-postgres",
            Self::Mysql => "sqlx-mysql",
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
        .replace("{{CLI_VERSION}}", env!("CARGO_PKG_VERSION"))
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
            crate::migration::steps::generate::lib_file::initial_lib().into_bytes(),
        ),
    ]
}

/// Writes the migration crate as one batch: all three files, or none.
pub(crate) fn write(migration_dir: &Path, engine: Engine, runique_version: &str) -> Result<()> {
    let mut batch = AtomicBuilder::create().journal(migration_dir);
    for (path, content) in files(migration_dir, engine, runique_version) {
        batch = batch.add(path, SourceItems::Bytes(content), false)?;
    }
    batch.start()?;
    Ok(())
}

/// Repairs a run the power cut short in `dir`, before the command reads
/// anything there. When files changed since, the repair touches nothing: they
/// are listed for the user to put right.
pub(crate) fn repair(dir: &Path) -> Result<()> {
    repaired(dir, crate::atomic::recover(dir))
}

/// [`repair`] for the migrations folder, whose runs may also have been
/// rewriting entity files (`migration up` taking out a `renamed_from`).
pub(crate) fn repair_beside(dir: &Path, entities: &Path) -> Result<()> {
    repaired(dir, crate::atomic::recover_beside(dir, Some(entities)))
}

fn repaired(dir: &Path, outcome: std::io::Result<bool>) -> Result<()> {
    let repaired = outcome.with_context(|| {
        tf(
            runique_i18n::keys::cli::error::INTERRUPTED_RUN,
            &[dir.display()],
        )
    })?;
    if repaired {
        println!(
            "{}",
            tf(
                runique_i18n::keys::cli::info::RUN_REPAIRED,
                &[dir.display()]
            )
        );
    }
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
    // A crate half written when the power went is taken back first: its folder
    // would otherwise pass for one that may hold migrations.
    repair(migration_dir)?;
    if migration_dir.exists() {
        return Ok(());
    }
    let engine = Engine::from_env().unwrap_or_else(|| {
        println!(
            "{}",
            tf(
                runique_i18n::keys::migrate::error::ENGINE_DEFAULTED,
                &[migration_dir.join("Cargo.toml").display()]
            )
        );
        Engine::Sqlite
    });
    write(migration_dir, engine, crate::tooling::VERSION)?;
    println!(
        "{}",
        tf(
            runique_i18n::keys::migrate::info::CRATE_CREATED,
            &[migration_dir.display()]
        )
    );
    println!("{}", t(runique_i18n::keys::makemigrations::hint::APPLY));
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

    // The crate's `main.rs` gives `run_cli` the project's migrator inside
    // Runique's: without it, a launch that fails stays half applied.
    #[test]
    fn the_crate_runs_its_migrator_through_runique_s() {
        let dir = std::env::temp_dir().join(format!("rq_mig_{}", uuid::Uuid::new_v4()));
        write(&dir, Engine::Sqlite, "9.8.7").unwrap();
        let main = fs::read_to_string(dir.join("src/main.rs")).unwrap();
        let _ = fs::remove_dir_all(&dir);
        assert!(
            main.contains(
                "cli::run_cli(runique_cli::migration::runtime::RuniqueMigrator(migration::Migrator))"
            ),
            "{main}"
        );
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
        let docs = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs");
        let row = format!("| {} | `{SEA_ORM_REQ}` |", crate::tooling::VERSION);
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

    // A `Cargo.toml` is already there: the crate is not written around it.
    #[test]
    fn the_crate_is_written_whole_or_not_at_all() {
        let root = crate::atomic::tests::TempDir::new();
        let dir = root.join("migration");
        fs::create_dir(&dir).unwrap();
        fs::write(dir.join("Cargo.toml"), "# mine").unwrap();

        assert!(write(&dir, Engine::Sqlite, "9.8.7").is_err());

        assert_eq!(
            fs::read_to_string(dir.join("Cargo.toml")).unwrap(),
            "# mine"
        );
        assert_eq!(root.entries(), ["migration", "migration/Cargo.toml"]);
    }

    #[test]
    fn the_crate_leaves_none_of_the_batch_s_own_files() {
        let root = crate::atomic::tests::TempDir::new();

        write(&root.join("migration"), Engine::Sqlite, "9.8.7").unwrap();

        assert_eq!(
            root.entries(),
            [
                "migration",
                "migration/Cargo.toml",
                "migration/src",
                "migration/src/lib.rs",
                "migration/src/main.rs"
            ]
        );
    }

    // The power went while the crate was being written: a folder with half of
    // it is left, which `ensure` would take for one holding migrations. The
    // journal says otherwise: it is taken back, then written whole.
    #[test]
    fn a_crate_cut_short_is_taken_back_then_written_whole() {
        let root = crate::atomic::tests::TempDir::new();
        let dir = root.join("migration");
        fs::create_dir_all(dir.join("src")).unwrap();
        fs::write(dir.join("Cargo.toml"), "placed").unwrap();
        fs::write(dir.join("Cargo.toml.runique-tmp"), "placed").unwrap();
        fs::write(dir.join("src/main.rs.runique-tmp"), "half").unwrap();
        crate::atomic::tests::cut_journal(
            &dir,
            false,
            &[
                ("Cargo.toml", false),
                ("src/main.rs", false),
                ("src/lib.rs", false),
            ],
            &["", "src"],
        );

        ensure(dir.join("src").to_str().unwrap()).unwrap();

        let cargo = fs::read_to_string(dir.join("Cargo.toml")).unwrap();
        assert!(cargo.contains("sea-orm-migration"), "{cargo}");
        assert_eq!(
            root.entries(),
            [
                "migration",
                "migration/Cargo.toml",
                "migration/src",
                "migration/src/lib.rs",
                "migration/src/main.rs"
            ]
        );
    }

    /// A project's environment, as `key=value` pairs.
    fn engine_of(vars: &[(&str, &str)]) -> Option<Engine> {
        Engine::from_lookup(|key| {
            vars.iter()
                .find(|(name, _)| *name == key)
                .map(|(_, value)| value.to_string())
        })
    }

    #[test]
    fn the_engine_is_the_one_the_database_url_names() {
        for (url, engine) in [
            ("sqlite://data.db?mode=rwc", Engine::Sqlite),
            ("postgres://user:pass@localhost/app", Engine::Postgres),
            ("postgresql://user:pass@localhost/app", Engine::Postgres),
            ("mysql://user:pass@localhost/app", Engine::Mysql),
        ] {
            assert_eq!(engine_of(&[("DATABASE_URL", url)]), Some(engine), "{url}");
        }
    }

    // What `runique new` writes is both, in agreement. Switching to Postgres
    // by its URL alone, the keyword left as it was: the app follows the URL,
    // and so does the crate, which reads that same URL to run.
    #[test]
    fn the_database_url_wins_over_the_engine_keyword() {
        assert_eq!(
            engine_of(&[
                ("DB_ENGINE", "sqlite"),
                ("DATABASE_URL", "postgres://user:pass@localhost/app"),
            ]),
            Some(Engine::Postgres)
        );
    }

    #[test]
    fn without_a_database_url_the_engine_keyword_decides() {
        for (keyword, engine) in [
            ("sqlite", Engine::Sqlite),
            ("postgres", Engine::Postgres),
            ("postgresql", Engine::Postgres),
            ("mysql", Engine::Mysql),
            ("  Postgres ", Engine::Postgres),
            ("MYSQL", Engine::Mysql),
        ] {
            assert_eq!(
                engine_of(&[("DB_ENGINE", keyword)]),
                Some(engine),
                "{keyword}"
            );
        }
    }

    // Nothing set, or something set that names no engine: the caller falls
    // back on SQLite and says so.
    #[test]
    fn no_engine_is_guessed_from_what_names_none() {
        assert_eq!(engine_of(&[]), None);
        assert_eq!(engine_of(&[("DB_ENGINE", "oracle")]), None);
        assert_eq!(engine_of(&[("DB_ENGINE", "mariadb")]), None);
        assert_eq!(
            engine_of(&[("DATABASE_URL", "mariadb://localhost/app")]),
            None
        );
        assert_eq!(engine_of(&[("DB_ENGINE", "")]), None);
        assert_eq!(
            engine_of(&[("DATABASE_URL", "oracle://localhost/app")]),
            None
        );
    }

    // A URL whose start names no engine is incomplete: the keys take over, as
    // they do when the app connects.
    #[test]
    fn the_keyword_names_the_engine_when_the_url_names_none() {
        assert_eq!(
            engine_of(&[
                ("DATABASE_URL", "oracle://localhost/app"),
                ("DB_ENGINE", "postgres"),
            ]),
            Some(Engine::Postgres)
        );
        assert_eq!(
            engine_of(&[("DATABASE_URL", ""), ("DB_ENGINE", "mysql")]),
            Some(Engine::Mysql)
        );
        // A URL that names one still decides.
        assert_eq!(
            engine_of(&[
                ("DATABASE_URL", "sqlite://data.db"),
                ("DB_ENGINE", "postgres"),
            ]),
            Some(Engine::Sqlite)
        );
    }

    // Each engine gets the runique feature of its own name, and its driver.
    #[test]
    fn each_engine_of_the_app_has_its_feature_and_its_driver() {
        for (engine, feature, driver) in [
            (DatabaseEngine::SQLite, "sqlite", "sqlx-sqlite"),
            (DatabaseEngine::PostgreSQL, "postgres", "sqlx-postgres"),
            (DatabaseEngine::MySQL, "mysql", "sqlx-mysql"),
        ] {
            let engine = Engine::from(engine);
            assert_eq!((engine.feature(), engine.sqlx_feature()), (feature, driver));
        }
    }
}
