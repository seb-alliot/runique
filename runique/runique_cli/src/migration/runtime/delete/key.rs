use crate::migration::runtime::engine::{backend, server, sqlite, unsupported};
use sea_orm::{DbBackend, DbErr};
use sea_orm_migration::SchemaManager;

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
