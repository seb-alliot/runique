//! SQL dialect differences sea-orm doesn't paper over. Copied from
//! `runique::db::sql_dialect`: the part the commands use.
use crate::utils::aliases::ADb;

/// The type name of a `CAST(x AS …)` to text. MySQL rejects `TEXT` as a
/// `CAST` target (`CAST(x AS CHAR)` is the equivalent there); PostgreSQL and
/// SQLite both accept `TEXT`.
pub fn text_cast_type(db: &ADb) -> &'static str {
    text_cast_type_of(db.get_database_backend())
}

/// Same as [`text_cast_type`], from the backend alone.
pub fn text_cast_type_of(backend: sea_orm::DbBackend) -> &'static str {
    match backend {
        sea_orm::DbBackend::MySql => "CHAR",
        _ => "TEXT",
    }
}

#[cfg(test)]
mod tests {
    use super::text_cast_type_of;
    use sea_orm::DbBackend;

    // The copy answers as `runique` does, for every engine.
    #[test]
    fn the_text_type_is_the_one_runique_gives() {
        for backend in [DbBackend::MySql, DbBackend::Postgres, DbBackend::Sqlite] {
            assert_eq!(
                text_cast_type_of(backend),
                runique::db::sql_dialect::text_cast_type_of(backend),
                "{backend:?}"
            );
        }
    }
}
