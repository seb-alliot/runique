//! Foreign keys added to, removed from or renamed on an existing table.
use super::engine::{backend, unsupported};
use super::{server, sqlite};
use sea_orm::{DbBackend, DbErr};
use sea_orm_migration::SchemaManager;
use sea_query::ForeignKeyCreateStatement;

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
