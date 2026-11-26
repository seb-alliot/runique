use crate::migration::runtime::engine::{backend, sqlite, unsupported};
use sea_orm::{DbBackend, DbErr};
use sea_orm_migration::SchemaManager;

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
