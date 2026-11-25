//! SQLite: a foreign key can't be added to or removed from an existing table;
//! the table has to be rebuilt, which Runique doesn't do yet. Until it does,
//! the migration is refused before it changes anything. Left to run, removing
//! a key would stop on a panic of the SQL builder, and dropping the table the
//! key points to would, with `ON DELETE CASCADE`, delete the rows of the
//! table that stays.
use runique_i18n::tf;
use sea_orm::DbErr;

pub(super) fn key_change_refused(table: &str) -> Result<(), DbErr> {
    Err(DbErr::Custom(tf(
        runique_i18n::keys::migrate::error::SQLITE_KEY_CHANGE,
        &[table],
    )))
}
