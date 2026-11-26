//! The launch of `migration up`: every pending migration applied, or none.
use super::engine::unsupported;
use sea_orm::{ConnectionTrait, DbBackend, DbErr, TransactionTrait};
use sea_orm_migration::{
    IntoSchemaManagerConnection, MigrationTrait, MigratorTrait, MigratorTraitSelf,
};
use sea_query::DynIden;

/// A project's migrator with Runique's own `up`; everything else stays
/// SeaORM's. The `main.rs` of the `migration` crate hands it to `run_cli`.
pub struct RuniqueMigrator<M>(pub M);

#[sea_orm_migration::async_trait::async_trait]
impl<M: MigratorTrait + Send + Sync> MigratorTraitSelf for RuniqueMigrator<M> {
    fn migrations(&self) -> Vec<Box<dyn MigrationTrait>> {
        M::migrations()
    }

    fn migration_table_name(&self) -> DynIden {
        M::migration_table_name()
    }

    async fn up<'c, C>(&self, db: C, steps: Option<u32>) -> Result<(), DbErr>
    where
        C: IntoSchemaManagerConnection<'c>,
    {
        let db = db.into_database_executor();
        match db.get_database_backend() {
            DbBackend::Postgres | DbBackend::Sqlite => {
                let transaction = db.begin().await?;
                match M::up(&transaction, steps).await {
                    Ok(()) => transaction.commit().await,
                    Err(failure) => {
                        let _ = transaction.rollback().await;
                        Err(failure)
                    }
                }
            }
            DbBackend::MySql => M::up(db, steps).await,
            other => Err(unsupported(other)),
        }
    }
}

#[cfg(all(test, feature = "sqlite"))]
mod tests {
    use super::*;
    use sea_orm::{Database, DatabaseConnection};
    use sea_orm_migration::{MigrationName, SchemaManager};
    use sea_query::{Alias, ColumnDef, IntoIden, Order, Query, Table};

    const RECORDS: &str = "rq_own_records";

    struct Create {
        table: &'static str,
        fails: bool,
    }

    impl MigrationName for Create {
        fn name(&self) -> &str {
            self.table
        }
    }

    #[sea_orm_migration::async_trait::async_trait]
    impl MigrationTrait for Create {
        async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
            manager
                .create_table(
                    Table::create()
                        .table(Alias::new(self.table))
                        .col(ColumnDef::new(Alias::new("id")).integer().primary_key())
                        .to_owned(),
                )
                .await?;
            if self.fails {
                return Err(DbErr::Custom("second migration fails".to_owned()));
            }
            Ok(())
        }
    }

    /// Two migrations, recorded in a table of the project's own.
    struct Passing;

    impl MigratorTrait for Passing {
        fn migrations() -> Vec<Box<dyn MigrationTrait>> {
            vec![
                Box::new(Create {
                    table: "first",
                    fails: false,
                }),
                Box::new(Create {
                    table: "second",
                    fails: false,
                }),
            ]
        }

        fn migration_table_name() -> DynIden {
            Alias::new(RECORDS).into_iden()
        }
    }

    /// The same, the second failing once its table is created.
    struct Failing;

    impl MigratorTrait for Failing {
        fn migrations() -> Vec<Box<dyn MigrationTrait>> {
            vec![
                Box::new(Create {
                    table: "first",
                    fails: false,
                }),
                Box::new(Create {
                    table: "second",
                    fails: true,
                }),
            ]
        }

        fn migration_table_name() -> DynIden {
            Alias::new(RECORDS).into_iden()
        }
    }

    async fn sqlite() -> DatabaseConnection {
        Database::connect("sqlite::memory:").await.unwrap()
    }

    /// The tables that exist, SeaORM's default records table included, and
    /// what `RECORDS` holds.
    async fn left(db: &DatabaseConnection) -> (Vec<&'static str>, Vec<String>) {
        let manager = SchemaManager::new(db);
        let mut tables = Vec::new();
        for table in ["first", "second", "seaql_migrations"] {
            if manager.has_table(table).await.unwrap() {
                tables.push(table);
            }
        }
        let mut recorded = Vec::new();
        if manager.has_table(RECORDS).await.unwrap() {
            let select = Query::select()
                .column(Alias::new("version"))
                .from(Alias::new(RECORDS))
                .order_by(Alias::new("version"), Order::Asc)
                .to_owned();
            for row in db.query_all(&select).await.unwrap() {
                recorded.push(row.try_get::<String>("", "version").unwrap());
            }
        }
        (tables, recorded)
    }

    // Every migration of the project is applied, and recorded where the
    // project records them, not in SeaORM's default table.
    #[tokio::test]
    async fn a_launch_that_passes_applies_and_records_every_migration() {
        let db = sqlite().await;

        RuniqueMigrator(Passing).up(&db, None).await.unwrap();

        assert_eq!(
            left(&db).await,
            (
                vec!["first", "second"],
                vec!["first".to_owned(), "second".to_owned()]
            )
        );
    }

    // The second migration fails after creating its table: neither table
    // stays, nor the record of the first, nor the records table itself.
    #[tokio::test]
    async fn a_launch_that_fails_leaves_nothing() {
        let db = sqlite().await;

        let failure = RuniqueMigrator(Failing).up(&db, None).await;

        assert_eq!(
            failure.unwrap_err().to_string(),
            DbErr::Custom("second migration fails".to_owned()).to_string()
        );
        assert_eq!(left(&db).await, (Vec::new(), Vec::new()));
    }

    // What stays SeaORM's (`status`, `down`) reads the project's migrations
    // through the wrapper: both are listed, and the one applied is told apart.
    #[tokio::test]
    async fn the_project_s_migrations_are_listed_with_their_state() {
        let db = sqlite().await;
        let migrator = RuniqueMigrator(Passing);
        migrator.up(&db, Some(1)).await.unwrap();

        let names = |list: Vec<sea_orm_migration::Migration>| -> Vec<String> {
            list.iter().map(|m| m.name().to_owned()).collect()
        };
        assert_eq!(
            names(migrator.get_applied_migrations(&db).await.unwrap()),
            ["first"]
        );
        assert_eq!(
            names(migrator.get_pending_migrations(&db).await.unwrap()),
            ["second"]
        );
    }

    // The number of migrations asked for is the one applied.
    #[tokio::test]
    async fn only_the_steps_asked_for_are_applied() {
        let db = sqlite().await;

        RuniqueMigrator(Passing).up(&db, Some(1)).await.unwrap();

        assert_eq!(
            left(&db).await,
            (vec!["first"], vec!["first".to_owned()])
        );
    }
}
