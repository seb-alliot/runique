//! The migrations of an enum, launched on real databases through
//! `RuniqueMigrator`, what the `main.rs` of a project's `migration` crate
//! hands to `cli::run_cli`.
//!
//! Each migration here runs what the file `makemigrations` generates for
//! that engine holds: the first test checks it against the generator, call
//! by call. Then the whole life of an enum goes through two launches and
//! back, on PostgreSQL, MySQL and SQLite:
//!
//! 1. the enum is created, then its two tables;
//! 2. rows are written;
//! 3. in one launch, the enum is renamed, a value is renamed and one added;
//! 4. `down` takes the two launches back, last first.
//!
//! Under PostgreSQL a launch is one transaction: the second part is also
//! what tells whether a type can be renamed and given a value inside it,
//! and what the database keeps when the launch fails half way.
//!
//! Postgres/MySQL need `docker compose -f docker-compose.yml up -d` and their
//! URL in `.env.test`; without it, those tests return early.

use crate::helpers::{db, db_mysql, db_postgres};
use runique::sea_orm::{ConnectionTrait, DatabaseConnection, DbBackend, TransactionTrait};
use runique_cli::{
    db::DatabaseEngine,
    migration::{
        EnumChange, EnumChanges, EnumPlace, generate_enum_file,
        runtime::{
            RuniqueMigrator, add_enum_value, create_enum_type, drop_enum_type, rename_enum_type,
            rename_enum_value,
        },
    },
};
use sea_orm_migration::{MigratorTraitSelf, prelude::*};
use sea_query::{ExprTrait, extension::postgres::Type};
use serial_test::serial;

const GENRE: &str = "rq_em_genre";
const CATEGORY: &str = "rq_em_category";
const BOOKS: (&str, &str) = ("rq_em_books", "genre");
const MOODS: (&str, &str) = ("rq_em_moods", "value");
const PLACES: [(&str, &str); 2] = [BOOKS, MOODS];
const VALUES: [&str; 2] = ["Roman", "Policier"];
// Its own records table: `seaql_migrations` belongs to the other tests.
const RECORDS: &str = "rq_em_migrations";

fn is_postgres(db: &impl ConnectionTrait) -> bool {
    db.get_database_backend() == DbBackend::Postgres
}

/// Whether the engine a migration runs on keeps an enum as a type: what
/// `makemigrations` knows of the project's engine when it writes the file.
fn typed(manager: &SchemaManager) -> bool {
    manager.get_database_backend() == DbBackend::Postgres
}

// ── The four migrations, as the generated files hold them ───────────────────

struct CreateEnum;

impl MigrationName for CreateEnum {
    fn name(&self) -> &str {
        "m1_create_rq_em_genre_enum"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for CreateEnum {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        if typed(manager) {
            create_enum_type(manager, GENRE, &VALUES).await?;
        }
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        if typed(manager) {
            drop_enum_type(manager, GENRE).await?;
        }
        Ok(())
    }
}

/// The column of an enum as the file of its table writes it: the type under
/// PostgreSQL, text elsewhere.
fn enum_column(manager: &SchemaManager, name: &str) -> ColumnDef {
    if typed(manager) {
        ColumnDef::new_with_type(
            Alias::new(name),
            ColumnType::Enum {
                name: Alias::new(GENRE).into_iden(),
                variants: VALUES.iter().map(|v| Alias::new(*v).into_iden()).collect(),
            },
        )
    } else {
        ColumnDef::new(Alias::new(name)).string().to_owned()
    }
}

struct CreateTables;

impl MigrationName for CreateTables {
    fn name(&self) -> &str {
        "m2_create_rq_em_tables"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for CreateTables {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        for place in PLACES {
            manager
                .create_table(
                    Table::create()
                        .table(Alias::new(place.0))
                        .col(
                            ColumnDef::new(Alias::new("id"))
                                .integer()
                                .not_null()
                                .primary_key(),
                        )
                        .col(enum_column(manager, place.1))
                        .to_owned(),
                )
                .await?;
        }
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        for place in PLACES {
            manager
                .drop_table(Table::drop().table(Alias::new(place.0)).to_owned())
                .await?;
        }
        Ok(())
    }
}

struct RenameEnum;

impl MigrationName for RenameEnum {
    fn name(&self) -> &str {
        "m3_rename_rq_em_category_enum"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for RenameEnum {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        if typed(manager) {
            rename_enum_type(manager, GENRE, CATEGORY).await?;
        }
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        if typed(manager) {
            rename_enum_type(manager, CATEGORY, GENRE).await?;
        }
        Ok(())
    }
}

/// Renames a value on every engine, and adds one where the enum is a type.
/// `fails`: stops once both are done, as a later instruction would.
struct AlterValues {
    fails: bool,
}

impl MigrationName for AlterValues {
    fn name(&self) -> &str {
        "m4_alter_rq_em_category_enum"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for AlterValues {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        rename_enum_value(manager, CATEGORY, "Roman", "Novel", &PLACES).await?;
        if typed(manager) {
            add_enum_value(manager, CATEGORY, "Essai").await?;
        }
        if self.fails {
            return Err(DbErr::Custom("the launch fails here".to_owned()));
        }
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        rename_enum_value(manager, CATEGORY, "Novel", "Roman", &PLACES).await
    }
}

struct Passing;

#[async_trait::async_trait]
impl MigratorTrait for Passing {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(CreateEnum),
            Box::new(CreateTables),
            Box::new(RenameEnum),
            Box::new(AlterValues { fails: false }),
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
            Box::new(CreateEnum),
            Box::new(CreateTables),
            Box::new(RenameEnum),
            Box::new(AlterValues { fails: true }),
        ]
    }

    fn migration_table_name() -> DynIden {
        Alias::new(RECORDS).into_iden()
    }
}

// ── What the migrations above run is what the generator writes ──────────────

fn places() -> Vec<EnumPlace> {
    PLACES
        .iter()
        .map(|(table, column)| EnumPlace {
            table: (*table).to_owned(),
            column: (*column).to_owned(),
        })
        .collect()
}

#[test]
fn the_migrations_of_this_file_run_what_the_generated_files_hold() {
    let created = EnumChanges {
        name: GENRE.to_owned(),
        change: EnumChange::Created,
        values: VALUES.iter().map(|v| (*v).to_owned()).collect(),
        places: places(),
        ..EnumChanges::default()
    };
    let renamed = EnumChanges {
        name: CATEGORY.to_owned(),
        change: EnumChange::Renamed,
        renamed_from: Some(GENRE.to_owned()),
        ..created.clone()
    };
    let altered = EnumChanges {
        name: CATEGORY.to_owned(),
        renamed_values: vec![("Roman".to_owned(), "Novel".to_owned())],
        added_values: vec!["Essai".to_owned()],
        places: places(),
        ..EnumChanges::default()
    };
    let create = r#"create_enum_type(manager, "rq_em_genre", &["Roman", "Policier"]).await?;"#;
    let drop = r#"drop_enum_type(manager, "rq_em_genre").await?;"#;
    let rename = r#"rename_enum_type(manager, "rq_em_genre", "rq_em_category").await?;"#;
    let rename_back = r#"rename_enum_type(manager, "rq_em_category", "rq_em_genre").await?;"#;
    let value = r#"rename_enum_value(manager, "rq_em_category", "Roman", "Novel", &[("rq_em_books", "genre"), ("rq_em_moods", "value")]).await?;"#;
    let value_back = r#"rename_enum_value(manager, "rq_em_category", "Novel", "Roman", &[("rq_em_books", "genre"), ("rq_em_moods", "value")]).await?;"#;
    let add = r#"add_enum_value(manager, "rq_em_category", "Essai").await?;"#;

    for (engine, typed) in [
        (DatabaseEngine::PostgreSQL, true),
        (DatabaseEngine::MySQL, false),
        (DatabaseEngine::SQLite, false),
    ] {
        let files = [
            generate_enum_file(&created, &engine),
            generate_enum_file(&renamed, &engine),
            generate_enum_file(&altered, &engine),
        ];
        // What only an engine with enum types runs.
        for (file, call) in [
            (&files[0], create),
            (&files[0], drop),
            (&files[1], rename),
            (&files[1], rename_back),
            (&files[2], add),
        ] {
            assert_eq!(file.contains(call), typed, "{engine:?}: {call}\n{file}");
        }
        // What every engine runs.
        for call in [value, value_back] {
            assert!(files[2].contains(call), "{engine:?}: {call}\n{}", files[2]);
        }
        // And nothing else: one call per line above, no other.
        let calls: usize = files
            .iter()
            .map(|file| file.matches("runique_cli::migration::runtime::").count())
            .sum();
        assert_eq!(calls, if typed { 7 } else { 2 }, "{engine:?}");
    }
}

// ── On a real database ───────────────────────────────────────────────────────

async fn clean(db: &DatabaseConnection) {
    for table in [BOOKS.0, MOODS.0, RECORDS] {
        let drop = Table::drop()
            .table(Alias::new(table))
            .if_exists()
            .to_owned();
        db.execute(&drop).await.expect("drop a test table");
    }
    if is_postgres(db) {
        for name in [GENRE, CATEGORY] {
            let drop = Type::drop().if_exists().name(Alias::new(name)).to_owned();
            db.execute(&drop).await.expect("drop a test type");
        }
    }
}

/// Writes a row whose enum column is of the type `of` under PostgreSQL.
async fn insert<C: ConnectionTrait>(
    db: &C,
    place: (&str, &str),
    id: i32,
    value: &str,
    of: &str,
) -> Result<(), DbErr> {
    let stored = if is_postgres(db) {
        Expr::val(value).as_enum(Alias::new(of))
    } else {
        Expr::val(value)
    };
    db.execute(
        &Query::insert()
            .into_table(Alias::new(place.0))
            .columns([Alias::new("id"), Alias::new(place.1)])
            .values_panic([id.into(), stored])
            .to_owned(),
    )
    .await
    .map(|_| ())
}

/// The values a place holds, by row, read as text on every engine.
async fn stored(db: &DatabaseConnection, place: (&str, &str)) -> Vec<String> {
    let column = Expr::col(Alias::new(place.1));
    let read = if is_postgres(db) {
        column.cast_as(Alias::new("text"))
    } else {
        column
    };
    db.query_all(
        &Query::select()
            .expr(read)
            .from(Alias::new(place.0))
            .order_by(Alias::new("id"), Order::Asc)
            .to_owned(),
    )
    .await
    .expect("read the rows")
    .iter()
    .map(|row| row.try_get_by_index::<String>(0).expect("a text value"))
    .collect()
}

/// The migrations the records table holds, in order.
async fn recorded(db: &DatabaseConnection) -> Vec<String> {
    if !SchemaManager::new(db).has_table(RECORDS).await.unwrap() {
        return Vec::new();
    }
    let select = Query::select()
        .column(Alias::new("version"))
        .from(Alias::new(RECORDS))
        .order_by(Alias::new("version"), Order::Asc)
        .to_owned();
    db.query_all(&select)
        .await
        .unwrap()
        .iter()
        .map(|row| row.try_get::<String>("", "version").unwrap())
        .collect()
}

/// Whether PostgreSQL has a type of that name.
async fn has_type(db: &DatabaseConnection, name: &str) -> bool {
    db.query_one(
        &Query::select()
            .expr(Expr::val(1))
            .from((Alias::new("pg_catalog"), Alias::new("pg_type")))
            .and_where(Expr::col(Alias::new("typname")).eq(name))
            .to_owned(),
    )
    .await
    .expect("read the catalogue")
    .is_some()
}

/// The first launch: the enum, then its tables, then rows as an app writes
/// them.
async fn created_with_rows(db: &DatabaseConnection) {
    clean(db).await;
    RuniqueMigrator(Passing)
        .up(db, Some(2))
        .await
        .expect("the first launch");
    for (id, value) in ["Roman", "Policier", "Roman"].into_iter().enumerate() {
        insert(db, BOOKS, id as i32, value, GENRE)
            .await
            .expect("a row");
    }
    for (id, value) in ["Roman", "Policier"].into_iter().enumerate() {
        insert(db, MOODS, id as i32, value, GENRE)
            .await
            .expect("a row");
    }
}

/// The whole life of an enum through its migrations, the same on every
/// engine.
async fn scenario(db: &DatabaseConnection) {
    created_with_rows(db).await;
    assert_eq!(
        recorded(db).await,
        ["m1_create_rq_em_genre_enum", "m2_create_rq_em_tables"]
    );
    if is_postgres(db) {
        assert!(has_type(db, GENRE).await);
    }

    // One launch: the enum renamed, a value renamed, a value added.
    RuniqueMigrator(Passing)
        .up(db, None)
        .await
        .expect("the second launch");

    assert_eq!(recorded(db).await.len(), 4);
    assert_eq!(stored(db, BOOKS).await, ["Novel", "Policier", "Novel"]);
    assert_eq!(stored(db, MOODS).await, ["Novel", "Policier"]);
    if is_postgres(db) {
        assert!(has_type(db, CATEGORY).await && !has_type(db, GENRE).await);
    }
    // The launch is over: the added value can be stored, under the new name
    // of the enum.
    insert(db, MOODS, 10, "Essai", CATEGORY)
        .await
        .expect("the added value");
    insert(db, BOOKS, 10, "Novel", CATEGORY)
        .await
        .expect("the renamed value");

    // Back over the second launch, last migration first.
    RuniqueMigrator(Passing)
        .down(db, Some(2))
        .await
        .expect("back over the second launch");

    assert_eq!(
        recorded(db).await,
        ["m1_create_rq_em_genre_enum", "m2_create_rq_em_tables"]
    );
    assert_eq!(
        stored(db, BOOKS).await,
        ["Roman", "Policier", "Roman", "Roman"]
    );
    // A value that was added stays, and so does the row that holds it.
    assert_eq!(stored(db, MOODS).await, ["Roman", "Policier", "Essai"]);
    if is_postgres(db) {
        assert!(has_type(db, GENRE).await && !has_type(db, CATEGORY).await);
    }

    // Back over the first: the tables go, then the enum nothing uses.
    RuniqueMigrator(Passing)
        .down(db, Some(2))
        .await
        .expect("back over the first launch");

    assert!(recorded(db).await.is_empty());
    let manager = SchemaManager::new(db);
    assert!(!manager.has_table(BOOKS.0).await.unwrap());
    assert!(!manager.has_table(MOODS.0).await.unwrap());
    if is_postgres(db) {
        assert!(!has_type(db, GENRE).await && !has_type(db, CATEGORY).await);
    }
    clean(db).await;
}

#[tokio::test]
async fn sqlite_an_enum_goes_through_its_migrations_and_back() {
    scenario(&db::fresh_db().await).await;
}

#[tokio::test]
#[serial]
async fn postgres_an_enum_goes_through_its_migrations_and_back() {
    let Some(db) = db_postgres::connect().await else {
        return;
    };
    scenario(&db).await;
}

#[tokio::test]
#[serial]
async fn mysql_an_enum_goes_through_its_migrations_and_back() {
    let Some(db) = db_mysql::connect().await else {
        return;
    };
    scenario(&db).await;
}

// ── A launch that fails half way ─────────────────────────────────────────────

// PostgreSQL undoes what a failed launch did to an enum with the rest: the
// type keeps its name and its values, the rows their value, and neither
// migration is recorded.
#[tokio::test]
#[serial]
async fn postgres_a_failed_launch_leaves_the_enum_as_it_was() {
    let Some(db) = db_postgres::connect().await else {
        return;
    };
    created_with_rows(&db).await;

    let failure = RuniqueMigrator(Failing).up(&db, None).await;

    assert!(failure.is_err());
    assert!(has_type(&db, GENRE).await && !has_type(&db, CATEGORY).await);
    assert_eq!(stored(&db, BOOKS).await, ["Roman", "Policier", "Roman"]);
    assert_eq!(recorded(&db).await.len(), 2);
    // `Essai` was added, then taken back with the launch: it is refused.
    assert!(insert(&db, MOODS, 10, "Essai", GENRE).await.is_err());
    clean(&db).await;
}

// SQLite undoes the rows the same way.
#[tokio::test]
async fn sqlite_a_failed_launch_leaves_the_rows_as_they_were() {
    let db = db::fresh_db().await;
    created_with_rows(&db).await;

    let failure = RuniqueMigrator(Failing).up(&db, None).await;

    assert!(failure.is_err());
    assert_eq!(stored(&db, BOOKS).await, ["Roman", "Policier", "Roman"]);
    assert_eq!(stored(&db, MOODS).await, ["Roman", "Policier"]);
    assert_eq!(recorded(&db).await.len(), 2);
}

// MySQL validates each statement as it runs: the rows the failed launch
// wrote stay written, and the migration before the one that failed stays
// recorded. This is what its compensation is for.
#[tokio::test]
#[serial]
async fn mysql_a_failed_launch_keeps_what_it_already_wrote() {
    let Some(db) = db_mysql::connect().await else {
        return;
    };
    created_with_rows(&db).await;

    let failure = RuniqueMigrator(Failing).up(&db, None).await;

    assert!(failure.is_err());
    assert_eq!(stored(&db, BOOKS).await, ["Novel", "Policier", "Novel"]);
    assert_eq!(recorded(&db).await.len(), 3);
    clean(&db).await;
}

// ── A value added, then used before the launch is over ──────────────────────

// Under PostgreSQL a value added to a type can't be used in the transaction
// that added it. A launch is one transaction: a migration that adds a value
// and one that writes it can't go in the same launch.
#[tokio::test]
#[serial]
async fn postgres_a_value_added_in_a_launch_can_not_be_used_before_it_ends() {
    let Some(db) = db_postgres::connect().await else {
        return;
    };
    created_with_rows(&db).await;

    let transaction = db.begin().await.expect("open the launch");
    add_enum_value(&SchemaManager::new(&transaction), GENRE, "Essai")
        .await
        .expect("add the value");
    let used = insert(&transaction, MOODS, 10, "Essai", GENRE).await;
    transaction.rollback().await.expect("close the launch");

    let refusal = used.expect_err("the value is not usable yet").to_string();
    assert!(refusal.contains("Essai"), "{refusal}");
    assert_eq!(stored(&db, MOODS).await, ["Roman", "Policier"]);
    clean(&db).await;
}
