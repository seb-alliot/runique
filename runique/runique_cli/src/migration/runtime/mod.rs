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
pub use create::{add_enum_value, add_foreign_key, create_enum_type};
pub use delete::{drop_enum_type, drop_foreign_key};
pub use launch::RuniqueMigrator;
pub use rename::{rename_enum_type, rename_enum_value, rename_foreign_key};

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

    // Neither is creating one or giving it a value: the table of the enum
    // column is left exactly as it was.
    #[tokio::test]
    async fn creating_an_enum_and_adding_a_value_are_nothing_on_sqlite() {
        let db = sqlite().await;
        let manager = SchemaManager::new(&db);
        create_enum_type(&manager, "regime", &["Vegan", "Omnivore"])
            .await
            .unwrap();
        add_enum_value(&manager, "regime", "Halal").await.unwrap();
        assert!(manager.has_table("menus").await.unwrap());
        assert!(!manager.has_table("regime").await.unwrap());
    }

    // No object carries the name of an enum off PostgreSQL: renaming one
    // asks nothing of the engine.
    #[tokio::test]
    async fn renaming_an_enum_is_nothing_on_sqlite() {
        let db = sqlite().await;
        let manager = SchemaManager::new(&db);
        rename_enum_type(&manager, "regime", "diet").await.unwrap();
        assert!(manager.has_table("menus").await.unwrap());
    }

    /// Two tables with an enum column, text on SQLite, and their rows.
    async fn enum_places(db: &sea_orm::DatabaseConnection) {
        for (table, column, rows) in [
            ("rq_books", "genre", vec!["Roman", "Policier", "Roman"]),
            ("rq_moods", "value", vec!["Roman", "Essai"]),
        ] {
            db.execute(
                &Table::create()
                    .table(Alias::new(table))
                    .col(ColumnDef::new(Alias::new("id")).integer().primary_key())
                    .col(ColumnDef::new(Alias::new(column)).string())
                    .to_owned(),
            )
            .await
            .unwrap();
            for (id, value) in rows.into_iter().enumerate() {
                db.execute(
                    &sea_query::Query::insert()
                        .into_table(Alias::new(table))
                        .columns([Alias::new("id"), Alias::new(column)])
                        .values_panic([(id as i32).into(), value.into()])
                        .to_owned(),
                )
                .await
                .unwrap();
            }
        }
    }

    /// The values of `column` in `table`, by row.
    async fn stored(db: &sea_orm::DatabaseConnection, table: &str, column: &str) -> Vec<String> {
        db.query_all(
            &sea_query::Query::select()
                .column(Alias::new(column))
                .from(Alias::new(table))
                .order_by(Alias::new("id"), sea_query::Order::Asc)
                .to_owned(),
        )
        .await
        .unwrap()
        .iter()
        .map(|row| row.try_get_by_index::<String>(0).unwrap())
        .collect()
    }

    // Text columns: the rows of every place that hold the value get the new
    // one, the others stay; the same call the other way round puts it back.
    #[tokio::test]
    async fn renaming_a_value_updates_the_rows_of_every_place_on_sqlite() {
        let db = sqlite().await;
        enum_places(&db).await;
        let manager = SchemaManager::new(&db);
        let places = [("rq_books", "genre"), ("rq_moods", "value")];

        rename_enum_value(&manager, "genre", "Roman", "Novel", &places)
            .await
            .unwrap();
        assert_eq!(
            stored(&db, "rq_books", "genre").await,
            ["Novel", "Policier", "Novel"]
        );
        assert_eq!(stored(&db, "rq_moods", "value").await, ["Novel", "Essai"]);

        rename_enum_value(&manager, "genre", "Novel", "Roman", &places)
            .await
            .unwrap();
        assert_eq!(
            stored(&db, "rq_books", "genre").await,
            ["Roman", "Policier", "Roman"]
        );
        assert_eq!(stored(&db, "rq_moods", "value").await, ["Roman", "Essai"]);
    }

    // Only the places given are written to: a column of the same name
    // elsewhere keeps its rows.
    #[tokio::test]
    async fn a_place_that_is_not_given_keeps_its_rows() {
        let db = sqlite().await;
        enum_places(&db).await;
        let manager = SchemaManager::new(&db);

        rename_enum_value(
            &manager,
            "genre",
            "Roman",
            "Novel",
            &[("rq_books", "genre")],
        )
        .await
        .unwrap();
        assert_eq!(stored(&db, "rq_moods", "value").await, ["Roman", "Essai"]);

        // No place at all: nothing to write, and no error.
        rename_enum_value(&manager, "genre", "Essai", "Essay", &[])
            .await
            .unwrap();
        assert_eq!(stored(&db, "rq_moods", "value").await, ["Roman", "Essai"]);
    }

    // A place that doesn't exist stops the migration: the error is the
    // engine's, not a silence.
    #[tokio::test]
    async fn a_place_that_does_not_exist_is_an_error() {
        let db = sqlite().await;
        let manager = SchemaManager::new(&db);
        assert!(
            rename_enum_value(
                &manager,
                "genre",
                "Roman",
                "Novel",
                &[("rq_missing", "genre")]
            )
            .await
            .is_err()
        );
    }
}
