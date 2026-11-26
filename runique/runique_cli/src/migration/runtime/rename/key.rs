use crate::migration::runtime::engine::{backend, server, unsupported};
use sea_orm::{DbBackend, DbErr};
use sea_orm_migration::SchemaManager;
use sea_query::ForeignKeyCreateStatement;

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
