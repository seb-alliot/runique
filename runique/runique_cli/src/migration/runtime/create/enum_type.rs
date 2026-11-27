use crate::migration::runtime::engine::{backend, postgres, unsupported};
use sea_orm::{DbBackend, DbErr};
use sea_orm_migration::SchemaManager;

/// Creates the enum `name` with `values`, unless it exists already. Only
/// PostgreSQL keeps an enum as an object of its own: elsewhere an enum
/// column is plain text and there is nothing to create.
pub async fn create_enum_type(
    manager: &SchemaManager<'_>,
    name: &str,
    values: &[&str],
) -> Result<(), DbErr> {
    match backend(manager) {
        DbBackend::Postgres => postgres::create_enum_type_if_missing(manager, name, values).await,
        DbBackend::MySql | DbBackend::Sqlite => Ok(()),
        other => Err(unsupported(other)),
    }
}
