//! Trial: one transaction given to SeaORM's migrator in place of the
//! connection, around a whole launch of two migrations.
//!
//! The second migration creates its table, then fails. What must be known,
//! per engine, is what the database keeps once the transaction is rolled
//! back: nothing under PostgreSQL and SQLite, which undo schema changes;
//! under MySQL each schema statement validates itself, so the launch stays
//! half applied. That difference is why MySQL gets a compensation instead.
//!
//! Second part: the same launch through `RuniqueMigrator`, what the
//! `main.rs` of a project's `migration` crate hands to `cli::run_cli`. It is
//! given the connection, as `run_cli` gives it, and opens the transaction
//! itself.
//!
//! Postgres/MySQL need `docker compose -f docker-compose.yml up -d` and their
//! URL in `.env.test`; without it, those tests return early.

use crate::helpers::{db, db_mysql, db_postgres};
use runique::sea_orm::{DatabaseConnection, TransactionTrait};
use runique_cli::migration::runtime::RuniqueMigrator;
use sea_orm_migration::MigratorTraitSelf;
use sea_orm_migration::prelude::*;
use serial_test::serial;

const FIRST: &str = "rq_launch_first";
const SECOND: &str = "rq_launch_second";
// Its own records table: `seaql_migrations` belongs to the other tests.
const RECORDS: &str = "rq_launch_migrations";

const FIRST_NAME: &str = "m1_rq_launch_first";
const SECOND_NAME: &str = "m2_rq_launch_second";

fn create(table: &str) -> TableCreateStatement {
    Table::create()
        .table(Alias::new(table))
        .col(
            ColumnDef::new(Alias::new("id"))
                .integer()
                .not_null()
                .primary_key(),
        )
        .to_owned()
}

struct CreateFirst;

impl MigrationName for CreateFirst {
    fn name(&self) -> &str {
        FIRST_NAME
    }
}

#[async_trait::async_trait]
impl MigrationTrait for CreateFirst {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager.create_table(create(FIRST)).await
    }
}

struct CreateSecond {
    fails: bool,
}

impl MigrationName for CreateSecond {
    fn name(&self) -> &str {
        SECOND_NAME
    }
}

#[async_trait::async_trait]
impl MigrationTrait for CreateSecond {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager.create_table(create(SECOND)).await?;
        if self.fails {
            return Err(DbErr::Custom("second migration fails".to_owned()));
        }
        Ok(())
    }
}

struct Passing;

#[async_trait::async_trait]
impl MigratorTrait for Passing {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(CreateFirst),
            Box::new(CreateSecond { fails: false }),
        ]
    }

    fn migration_table_name() -> DynIden {
        Alias::new(RECORDS).into_iden()
    }
}

struct Failing;

#[async_trait::async_trait]
impl MigratorTrait for Failing {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(CreateFirst),
            Box::new(CreateSecond { fails: true }),
        ]
    }

    fn migration_table_name() -> DynIden {
        Alias::new(RECORDS).into_iden()
    }
}

/// The whole launch in one transaction: validated when every migration
/// passed, rolled back as soon as one fails.
async fn launch<M: MigratorTrait>(db: &DatabaseConnection) -> Result<(), DbErr> {
    let transaction = db.begin().await?;
    match M::up(&transaction, None).await {
        Ok(()) => transaction.commit().await,
        Err(failure) => {
            transaction.rollback().await?;
            Err(failure)
        }
    }
}

/// Who opens the transaction: the test, or the migrator given the connection.
#[derive(Clone, Copy)]
enum By {
    Transaction,
    Migrator,
}

async fn run<M: MigratorTrait + Send + Sync>(
    by: By,
    migrator: M,
    db: &DatabaseConnection,
) -> Result<(), DbErr> {
    match by {
        By::Transaction => launch::<M>(db).await,
        By::Migrator => RuniqueMigrator(migrator).up(db, None).await,
    }
}

async fn clean(db: &DatabaseConnection) {
    for table in [SECOND, FIRST, RECORDS] {
        let drop = Table::drop()
            .table(Alias::new(table))
            .if_exists()
            .to_owned();
        db.execute(&drop).await.expect("drop a test table");
    }
}

/// What the database holds after a launch.
#[derive(Debug, PartialEq)]
struct Left {
    first: bool,
    second: bool,
    recorded: Vec<String>,
}

async fn left(db: &DatabaseConnection) -> Left {
    let manager = SchemaManager::new(db);
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
    Left {
        first: manager.has_table(FIRST).await.unwrap(),
        second: manager.has_table(SECOND).await.unwrap(),
        recorded,
    }
}

fn nothing() -> Left {
    Left {
        first: false,
        second: false,
        recorded: Vec::new(),
    }
}

fn everything() -> Left {
    Left {
        first: true,
        second: true,
        recorded: vec![FIRST_NAME.to_owned(), SECOND_NAME.to_owned()],
    }
}

async fn after_a_failed_launch(by: By, db: &DatabaseConnection) -> Left {
    clean(db).await;
    let failure = run(by, Failing, db).await;
    let state = left(db).await;
    clean(db).await;
    assert!(
        failure.is_err(),
        "the second migration must fail the launch"
    );
    state
}

async fn after_a_launch_that_passes(by: By, db: &DatabaseConnection) -> Left {
    clean(db).await;
    let done = run(by, Passing, db).await;
    let state = left(db).await;
    clean(db).await;
    assert!(done.is_ok(), "{done:?}");
    state
}

// ── A launch that fails ──────────────────────────────────────────────────────

#[tokio::test]
#[serial]
async fn a_failed_launch_leaves_nothing_on_postgres() {
    let Some(db) = db_postgres::connect().await else {
        return;
    };
    assert_eq!(after_a_failed_launch(By::Transaction, &db).await, nothing());
}

#[tokio::test]
async fn a_failed_launch_leaves_nothing_on_sqlite() {
    let db = db::fresh_db().await;
    assert_eq!(after_a_failed_launch(By::Transaction, &db).await, nothing());
}

// MySQL validates each schema statement by itself: the transaction protects
// nothing, both tables stay and the first migration stays recorded.
#[tokio::test]
#[serial]
async fn a_failed_launch_stays_half_applied_on_mysql() {
    let Some(db) = db_mysql::connect().await else {
        return;
    };
    assert_eq!(
        after_a_failed_launch(By::Transaction, &db).await,
        Left {
            first: true,
            second: true,
            recorded: vec![FIRST_NAME.to_owned()],
        }
    );
}

// ── A launch that passes ─────────────────────────────────────────────────────

#[tokio::test]
#[serial]
async fn a_launch_that_passes_is_kept_on_postgres() {
    let Some(db) = db_postgres::connect().await else {
        return;
    };
    assert_eq!(
        after_a_launch_that_passes(By::Transaction, &db).await,
        everything()
    );
}

#[tokio::test]
async fn a_launch_that_passes_is_kept_on_sqlite() {
    let db = db::fresh_db().await;
    assert_eq!(
        after_a_launch_that_passes(By::Transaction, &db).await,
        everything()
    );
}

#[tokio::test]
#[serial]
async fn a_launch_that_passes_is_kept_on_mysql() {
    let Some(db) = db_mysql::connect().await else {
        return;
    };
    assert_eq!(
        after_a_launch_that_passes(By::Transaction, &db).await,
        everything()
    );
}

// ── The same, through `RuniqueMigrator` ──────────────────────────────────────

#[tokio::test]
#[serial]
async fn the_migrator_leaves_nothing_of_a_failed_launch_on_postgres() {
    let Some(db) = db_postgres::connect().await else {
        return;
    };
    assert_eq!(after_a_failed_launch(By::Migrator, &db).await, nothing());
}

#[tokio::test]
async fn the_migrator_leaves_nothing_of_a_failed_launch_on_sqlite() {
    let db = db::fresh_db().await;
    assert_eq!(after_a_failed_launch(By::Migrator, &db).await, nothing());
}

// No transaction on MySQL: what SeaORM does today, until the compensation.
#[tokio::test]
#[serial]
async fn the_migrator_leaves_a_failed_launch_half_applied_on_mysql() {
    let Some(db) = db_mysql::connect().await else {
        return;
    };
    assert_eq!(
        after_a_failed_launch(By::Migrator, &db).await,
        Left {
            first: true,
            second: true,
            recorded: vec![FIRST_NAME.to_owned()],
        }
    );
}

#[tokio::test]
#[serial]
async fn the_migrator_keeps_a_launch_that_passes_on_postgres() {
    let Some(db) = db_postgres::connect().await else {
        return;
    };
    assert_eq!(
        after_a_launch_that_passes(By::Migrator, &db).await,
        everything()
    );
}

#[tokio::test]
async fn the_migrator_keeps_a_launch_that_passes_on_sqlite() {
    let db = db::fresh_db().await;
    assert_eq!(
        after_a_launch_that_passes(By::Migrator, &db).await,
        everything()
    );
}

#[tokio::test]
#[serial]
async fn the_migrator_keeps_a_launch_that_passes_on_mysql() {
    let Some(db) = db_mysql::connect().await else {
        return;
    };
    assert_eq!(
        after_a_launch_that_passes(By::Migrator, &db).await,
        everything()
    );
}
