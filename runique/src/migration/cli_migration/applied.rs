//! What the database already applied: the migrations `seaql_migrations`
//! records, and the checks `makemigrations` runs on them before computing.
use super::lib_file::lib_modules;
use crate::auth::migrations_table::FRAMEWORK_MIGRATION_NAMES;
use crate::utils::trad::tf;
use anyhow::Result;
use sea_orm::{DatabaseConnection, EntityTrait, QueryOrder};
use sea_orm_migration::{SchemaManager, seaql_migrations};

/// The migrations `seaql_migrations` records, oldest first. Empty when the
/// table doesn't exist: nothing was ever applied to this database.
pub async fn read(db: &DatabaseConnection) -> Result<Vec<String>> {
    if !SchemaManager::new(db).has_table("seaql_migrations").await? {
        return Ok(Vec::new());
    }
    let rows = seaql_migrations::Entity::find()
        .order_by_asc(seaql_migrations::Column::Version)
        .all(db)
        .await?;
    Ok(rows.into_iter().map(|row| row.version).collect())
}

/// Refuses a database that holds a migration this project doesn't know:
/// neither one of the framework's, nor a module of `lib.rs`.
pub(crate) fn check_known(applied: &[String], migrations_path: &str) -> Result<()> {
    let known = lib_modules(migrations_path)?;
    let unknown: Vec<&str> = applied
        .iter()
        .map(String::as_str)
        .filter(|name| {
            !FRAMEWORK_MIGRATION_NAMES.contains(name) && !known.iter().any(|m| m == name)
        })
        .collect();
    if unknown.is_empty() {
        return Ok(());
    }
    anyhow::bail!(
        "{}",
        tf("makemigrations.applied_unknown", &[unknown.join("\n  ")])
    )
}

#[cfg(all(test, feature = "sqlite"))]
mod tests {
    use super::*;
    use crate::atomic::tests::TempDir;
    use crate::migration::cli_migration::{initial_lib, makemigrations_with};
    use sea_orm::{ActiveModelTrait, ConnectionTrait, Database, DbBackend, Schema, Set};
    use std::fs;

    const OWN: &str = "m20260101_000000_create_notes_table";

    /// A database whose `seaql_migrations` records `versions`.
    async fn database(url: &str, versions: &[&str]) -> DatabaseConnection {
        let db = Database::connect(url).await.unwrap();
        let table =
            Schema::new(DbBackend::Sqlite).create_table_from_entity(seaql_migrations::Entity);
        db.execute(&table).await.unwrap();
        for version in versions {
            seaql_migrations::ActiveModel {
                version: Set(version.to_string()),
                applied_at: Set(0),
            }
            .insert(&db)
            .await
            .unwrap();
        }
        db
    }

    /// A `migration/src` with the framework's `lib.rs`, plus `OWN` when asked.
    fn migrations_src(dir: &TempDir, with_own: bool) -> String {
        let src = dir.join("src");
        fs::create_dir_all(&src).unwrap();
        let mut lib = initial_lib();
        if with_own {
            lib = format!("mod {OWN};\n{lib}");
        }
        fs::write(src.join("lib.rs"), lib).unwrap();
        src.to_str().unwrap().to_string()
    }

    #[tokio::test]
    async fn a_database_without_the_table_has_applied_nothing() {
        let db = Database::connect("sqlite::memory:").await.unwrap();
        assert!(read(&db).await.unwrap().is_empty());
    }

    #[tokio::test]
    async fn the_recorded_migrations_are_read_oldest_first() {
        let db = database("sqlite::memory:", &[OWN, FRAMEWORK_MIGRATION_NAMES[0]]).await;
        assert_eq!(
            read(&db).await.unwrap(),
            [FRAMEWORK_MIGRATION_NAMES[0], OWN]
        );
    }

    #[test]
    fn framework_migrations_and_lib_modules_are_known() {
        let dir = TempDir::new();
        let path = migrations_src(&dir, true);
        let mut applied: Vec<String> = FRAMEWORK_MIGRATION_NAMES.map(String::from).to_vec();
        applied.push(OWN.to_string());

        check_known(&applied, &path).unwrap();
    }

    #[test]
    fn a_migration_missing_from_lib_is_refused_by_name() {
        let dir = TempDir::new();
        let path = migrations_src(&dir, false);

        let error = check_known(&[OWN.to_string()], &path).unwrap_err();
        assert!(error.to_string().contains(OWN), "{error}");
    }

    // No `lib.rs` at all: only the framework's own migrations are known.
    #[test]
    fn without_a_lib_only_the_framework_migrations_are_known() {
        let dir = TempDir::new();
        let path = dir.join("src").to_str().unwrap().to_string();
        let framework: Vec<String> = FRAMEWORK_MIGRATION_NAMES.map(String::from).to_vec();

        check_known(&framework, &path).unwrap();
        assert!(check_known(&[OWN.to_string()], &path).is_err());
    }

    // The whole command: the refusal comes before anything is written.
    #[tokio::test]
    async fn the_command_refuses_an_unknown_applied_migration_and_writes_nothing() {
        let (project, base) = (TempDir::new(), TempDir::new());
        let path = migrations_src(&project, false);
        let entities = project.join("entities");
        fs::create_dir_all(&entities).unwrap();
        let url = format!("sqlite://{}?mode=rwc", base.join("base.sqlite").display());
        drop(database(&url, &[OWN]).await);
        let on_disk = project.entries();

        let get = |key: &str| (key == "DATABASE_URL").then(|| url.clone());
        let error = makemigrations_with(get, entities.to_str().unwrap(), &path, false)
            .await
            .unwrap_err();

        assert!(error.to_string().contains(OWN), "{error}");
        assert_eq!(project.entries(), on_disk);
    }
}
