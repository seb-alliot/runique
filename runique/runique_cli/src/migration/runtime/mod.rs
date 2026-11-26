//! What a generated migration hands over to Runique while it runs: the
//! operations the engines don't do the same way. The file only names the
//! operation; the engine is read on the open connection, so a migration
//! written once behaves as each engine needs, and gains what Runique learns
//! later without being generated again.
mod check;
mod create;
mod delete;
mod engine;
mod launch;
mod rename;

pub use check::key_changes_supported;
pub use create::add_foreign_key;
pub use delete::{drop_enum_type, drop_foreign_key};
pub use launch::RuniqueMigrator;
pub use rename::rename_foreign_key;

#[cfg(all(test, feature = "sqlite"))]
mod tests {
    use super::*;
    use runique_i18n::tf;
    use sea_orm::{ConnectionTrait, Database, DbErr};
    use sea_orm_migration::SchemaManager;
    use sea_query::{Alias, ColumnDef, ForeignKey, Table};

    /// SQLite with a `menus` table.
    async fn sqlite() -> sea_orm::DatabaseConnection {
        let db = Database::connect("sqlite::memory:").await.unwrap();
        db.execute(
            &Table::create()
                .table(Alias::new("menus"))
                .col(ColumnDef::new(Alias::new("id")).integer().primary_key())
                .to_owned(),
        )
        .await
        .unwrap();
        db
    }

    // On SQLite the three entry points refuse, with the same message that
    // names the table, and nothing is run: the library would panic otherwise.
    #[tokio::test]
    async fn sqlite_refuses_a_key_change_on_an_existing_table() {
        let db = sqlite().await;
        let manager = SchemaManager::new(&db);
        let refusal = tf(
            runique_i18n::keys::migrate::error::SQLITE_KEY_CHANGE,
            &["menus"],
        );
        let key = ForeignKey::create()
            .name("menus_cat_id_categories_fkey")
            .from(Alias::new("menus"), Alias::new("cat_id"))
            .to(Alias::new("categories"), Alias::new("id"))
            .to_owned();

        assert_eq!(
            key_changes_supported(&manager, "menus")
                .unwrap_err()
                .to_string(),
            DbErr::Custom(refusal.clone()).to_string()
        );
        assert_eq!(
            drop_foreign_key(&manager, "menus", "menus_cat_id_categories_fkey")
                .await
                .unwrap_err()
                .to_string(),
            DbErr::Custom(refusal.clone()).to_string()
        );
        assert_eq!(
            add_foreign_key(&manager, "menus", key)
                .await
                .unwrap_err()
                .to_string(),
            DbErr::Custom(refusal).to_string()
        );
        assert!(manager.has_table("menus").await.unwrap());
    }

    // No enum type off PostgreSQL: nothing is asked of the engine.
    #[tokio::test]
    async fn dropping_an_enum_type_is_nothing_on_sqlite() {
        let db = sqlite().await;
        drop_enum_type(&SchemaManager::new(&db), "regime")
            .await
            .unwrap();
    }
}
