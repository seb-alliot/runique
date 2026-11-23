//! What a generated migration hands over to Runique while it runs: the
//! operations the engines don't do the same way. The file only names the
//! operation; the engine is read on the open connection, so a migration
//! written once behaves as each engine needs, and gains what Runique learns
//! later without being generated again.
mod postgres;
mod server;
mod sqlite;

use crate::utils::trad::tf;
use sea_orm::{ConnectionTrait, DbBackend, DbErr};
use sea_orm_migration::SchemaManager;
use sea_query::ForeignKeyCreateStatement;

fn backend(manager: &SchemaManager<'_>) -> DbBackend {
    manager.get_connection().get_database_backend()
}

fn unsupported(backend: DbBackend) -> DbErr {
    DbErr::Custom(tf(
        runique_i18n::keys::migrate::error::ENGINE_UNSUPPORTED,
        &[format!("{backend:?}")],
    ))
}

/// Called first by a migration that adds or removes a foreign key on an
/// existing table, before it changes anything: an engine that can't do it
/// stops the migration while the table is still as it was.
pub fn key_changes_supported(manager: &SchemaManager<'_>, table: &str) -> Result<(), DbErr> {
    match backend(manager) {
        DbBackend::Postgres | DbBackend::MySql => Ok(()),
        DbBackend::Sqlite => sqlite::key_change_refused(table),
        other => Err(unsupported(other)),
    }
}

/// Removes the foreign key `name` of the existing table `table`.
pub async fn drop_foreign_key(
    manager: &SchemaManager<'_>,
    table: &str,
    name: &str,
) -> Result<(), DbErr> {
    match backend(manager) {
        DbBackend::Postgres | DbBackend::MySql => {
            server::drop_foreign_key(manager, table, name).await
        }
        DbBackend::Sqlite => sqlite::key_change_refused(table),
        other => Err(unsupported(other)),
    }
}

/// Adds the foreign key `key` to the existing table `table`.
pub async fn add_foreign_key(
    manager: &SchemaManager<'_>,
    table: &str,
    key: ForeignKeyCreateStatement,
) -> Result<(), DbErr> {
    match backend(manager) {
        DbBackend::Postgres | DbBackend::MySql => server::add_foreign_key(manager, key).await,
        DbBackend::Sqlite => sqlite::key_change_refused(table),
        other => Err(unsupported(other)),
    }
}

/// Gives the foreign key `old_name` of `table` the name of `key`, which says
/// the same thing: after a table rename, the name still holds the former
/// table. PostgreSQL and MySQL have no statement that renames a key, it is
/// removed and added again. Under SQLite a key follows its table by itself
/// and its name is never used to reach it: nothing to do.
pub async fn rename_foreign_key(
    manager: &SchemaManager<'_>,
    table: &str,
    old_name: &str,
    key: ForeignKeyCreateStatement,
) -> Result<(), DbErr> {
    match backend(manager) {
        DbBackend::Postgres | DbBackend::MySql => {
            server::drop_foreign_key(manager, table, old_name).await?;
            server::add_foreign_key(manager, key).await
        }
        DbBackend::Sqlite => Ok(()),
        other => Err(unsupported(other)),
    }
}

/// Drops the enum type `name` once no column uses it any more. Only
/// PostgreSQL has such types: elsewhere an enum column is plain text and
/// there is nothing to drop.
pub async fn drop_enum_type(manager: &SchemaManager<'_>, name: &str) -> Result<(), DbErr> {
    match backend(manager) {
        DbBackend::Postgres => postgres::drop_enum_type_if_unused(manager, name).await,
        DbBackend::MySql | DbBackend::Sqlite => Ok(()),
        other => Err(unsupported(other)),
    }
}

#[cfg(all(test, feature = "sqlite"))]
mod tests {
    use super::*;
    use sea_orm::Database;
    use sea_query::{Alias, ColumnDef, ForeignKey, Table};

    /// SQLite with a `menus` table.
    async fn sqlite() -> sea_orm::DatabaseConnection {
        let db = Database::connect("sqlite::memory:").await.unwrap();
        db.execute(
            &Table::create()
                .table(Alias::new("menus"))
                .col(ColumnDef::new(Alias::new("id")).integer().primary_key())
                .to_owned(),
        )
        .await
        .unwrap();
        db
    }

    // On SQLite the three entry points refuse, with the same message that
    // names the table, and nothing is run: the library would panic otherwise.
    #[tokio::test]
    async fn sqlite_refuses_a_key_change_on_an_existing_table() {
        let db = sqlite().await;
        let manager = SchemaManager::new(&db);
        let refusal = tf(
            runique_i18n::keys::migrate::error::SQLITE_KEY_CHANGE,
            &["menus"],
        );
        let key = ForeignKey::create()
            .name("menus_cat_id_categories_fkey")
            .from(Alias::new("menus"), Alias::new("cat_id"))
            .to(Alias::new("categories"), Alias::new("id"))
            .to_owned();

        assert_eq!(
            key_changes_supported(&manager, "menus")
                .unwrap_err()
                .to_string(),
            DbErr::Custom(refusal.clone()).to_string()
        );
        assert_eq!(
            drop_foreign_key(&manager, "menus", "menus_cat_id_categories_fkey")
                .await
                .unwrap_err()
                .to_string(),
            DbErr::Custom(refusal.clone()).to_string()
        );
        assert_eq!(
            add_foreign_key(&manager, "menus", key)
                .await
                .unwrap_err()
                .to_string(),
            DbErr::Custom(refusal).to_string()
        );
        assert!(manager.has_table("menus").await.unwrap());
    }

    // No enum type off PostgreSQL: nothing is asked of the engine.
    #[tokio::test]
    async fn dropping_an_enum_type_is_nothing_on_sqlite() {
        let db = sqlite().await;
        drop_enum_type(&SchemaManager::new(&db), "regime")
            .await
            .unwrap();
    }
}
