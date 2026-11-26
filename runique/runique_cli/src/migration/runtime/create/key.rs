use crate::migration::runtime::engine::{backend, server, sqlite, unsupported};
use sea_orm::{DbBackend, DbErr};
use sea_orm_migration::SchemaManager;
use sea_query::ForeignKeyCreateStatement;

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
