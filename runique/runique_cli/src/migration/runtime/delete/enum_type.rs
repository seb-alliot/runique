use crate::migration::runtime::engine::{backend, postgres, unsupported};
use sea_orm::{DbBackend, DbErr};
use sea_orm_migration::SchemaManager;

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
