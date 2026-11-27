use crate::migration::runtime::engine::{backend, postgres, unsupported};
use sea_orm::{DbBackend, DbErr};
use sea_orm_migration::SchemaManager;

/// Gives the enum `from` the name `to`. Under PostgreSQL its type is renamed
/// in place: every column that uses it follows, their rows untouched.
/// Elsewhere an enum column is plain text and no object carries the name.
pub async fn rename_enum_type(
    manager: &SchemaManager<'_>,
    from: &str,
    to: &str,
) -> Result<(), DbErr> {
    match backend(manager) {
        DbBackend::Postgres => postgres::rename_enum_type(manager, from, to).await,
        DbBackend::MySql | DbBackend::Sqlite => Ok(()),
        other => Err(unsupported(other)),
    }
}
