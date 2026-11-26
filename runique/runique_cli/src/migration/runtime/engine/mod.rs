//! Which engine a migration runs on, read on its open connection.
pub(crate) mod postgres;
pub(crate) mod server;
pub(crate) mod sqlite;

use runique_i18n::tf;
use sea_orm::{ConnectionTrait, DbBackend, DbErr};
use sea_orm_migration::SchemaManager;

pub(crate) fn backend(manager: &SchemaManager<'_>) -> DbBackend {
    manager.get_connection().get_database_backend()
}

pub(crate) fn unsupported(backend: DbBackend) -> DbErr {
    DbErr::Custom(tf(
        runique_i18n::keys::migrate::error::ENGINE_UNSUPPORTED,
        &[format!("{backend:?}")],
    ))
}
