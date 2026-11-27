use crate::migration::runtime::engine::{backend, postgres, text, unsupported};
use sea_orm::{DbBackend, DbErr};
use sea_orm_migration::SchemaManager;

/// Renames the value `from` of the enum `name` to `to`, once for the enum
/// whatever the number of columns that use it. `places` are those columns,
/// as (table, column), named as they are when the migration runs.
///
/// Under PostgreSQL the value is renamed in the type, which every column
/// shares: the places are not touched. Under MySQL and SQLite the enum is
/// text in each column: the rows of each place that hold `from` get `to`.
pub async fn rename_enum_value(
    manager: &SchemaManager<'_>,
    name: &str,
    from: &str,
    to: &str,
    places: &[(&str, &str)],
) -> Result<(), DbErr> {
    match backend(manager) {
        DbBackend::Postgres => postgres::rename_enum_value(manager, name, from, to).await,
        DbBackend::MySql | DbBackend::Sqlite => {
            text::rename_enum_value(manager, places, from, to).await
        }
        other => Err(unsupported(other)),
    }
}
