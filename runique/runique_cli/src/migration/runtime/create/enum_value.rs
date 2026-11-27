use crate::migration::runtime::engine::{backend, postgres, unsupported};
use sea_orm::{DbBackend, DbErr};
use sea_orm_migration::SchemaManager;

/// Adds `value` to the enum `name`, unless it has it already. Under
/// PostgreSQL the type has to learn the value; elsewhere the column is text
/// and takes it as it is.
pub async fn add_enum_value(
    manager: &SchemaManager<'_>,
    name: &str,
    value: &str,
) -> Result<(), DbErr> {
    match backend(manager) {
        DbBackend::Postgres => postgres::add_enum_value(manager, name, value).await,
        DbBackend::MySql | DbBackend::Sqlite => Ok(()),
        other => Err(unsupported(other)),
    }
}
