//! A table renamed, the way a generated migration does it: the table, then
//! the index and the keys whose name held the former table name
//! (`migration::runtime::rename_foreign_key`), and the way back.
//!
//! On the three engines the rows stay and the keys keep working: the table's
//! own key, and the key of a table that points to it, which follows the
//! rename by itself. PostgreSQL and MySQL give the keys their new name by
//! removing and adding them; SQLite has nothing to do.
//!
//! Postgres/MySQL need `docker compose -f docker-compose.yml up -d` and their
//! URL in `.env.test`; without it, those tests return early.

use crate::helpers::{db, db_mysql, db_postgres};
use runique::migration::runtime::{drop_foreign_key, rename_foreign_key};
use runique::sea_orm::{ConnectionTrait, DatabaseConnection, DbBackend};
use sea_orm_migration::SchemaManager;
use sea_query::{Alias, ColumnDef, ForeignKey, ForeignKeyCreateStatement, Index, Query, Table};
use serial_test::serial;

const OWNER: &str = "rq_rn_owner";
const SHOP: &str = "rq_rn_shop";
const BAKERY: &str = "rq_rn_bakery";
const PRODUCT: &str = "rq_rn_product";

const SHOP_INDEX: &str = "idx_rq_rn_shop_city";
const BAKERY_INDEX: &str = "idx_rq_rn_bakery_city";
// The table's own key, and the key of the table that points to it.
const SHOP_KEY: &str = "rq_rn_shop_owner_id_rq_rn_owner_fkey";
const BAKERY_KEY: &str = "rq_rn_bakery_owner_id_rq_rn_owner_fkey";
const TO_SHOP_KEY: &str = "rq_rn_product_shop_id_rq_rn_shop_fkey";
const TO_BAKERY_KEY: &str = "rq_rn_product_shop_id_rq_rn_bakery_fkey";

fn key(name: &str, from: (&str, &str), to: &str) -> ForeignKeyCreateStatement {
    ForeignKey::create()
        .name(name)
        .from(Alias::new(from.0), Alias::new(from.1))
        .to(Alias::new(to), Alias::new("id"))
        .to_owned()
}

async fn drop_all(db: &DatabaseConnection) {
    for table in [PRODUCT, BAKERY, SHOP, OWNER] {
        let drop = Table::drop().table(Alias::new(table)).if_exists().to_owned();
        db.execute(&drop).await.expect("drop a test table");
    }
}

fn id() -> ColumnDef {
    ColumnDef::new(Alias::new("id"))
        .integer()
        .not_null()
        .primary_key()
        .to_owned()
}

async fn insert(
    db: &DatabaseConnection,
    table: &str,
    columns: &[&str],
    values: &[i32],
) -> Result<(), runique::sea_orm::DbErr> {
    let insert = Query::insert()
        .into_table(Alias::new(table))
        .columns(columns.iter().map(|c| Alias::new(*c)))
        .values_panic(values.iter().map(|v| (*v).into()))
        .to_owned();
    db.execute(&insert).await.map(|_| ())
}

async fn rows(db: &DatabaseConnection, table: &str) -> usize {
    let select = Query::select()
        .column(Alias::new("id"))
        .from(Alias::new(table))
        .to_owned();
    db.query_all(&select).await.expect("read a test table").len()
}

/// `owner`, `shop` (a key to `owner`, an index on `city`) and `product` (a
/// key to `shop`), as Runique creates them, with one row each.
async fn three_tables(db: &DatabaseConnection) {
    drop_all(db).await;
    let owner = Table::create().table(Alias::new(OWNER)).col(id()).to_owned();
    let shop = Table::create()
        .table(Alias::new(SHOP))
        .col(id())
        .col(ColumnDef::new(Alias::new("owner_id")).integer().not_null())
        .col(ColumnDef::new(Alias::new("city")).integer().not_null())
        .foreign_key(&mut key(SHOP_KEY, (SHOP, "owner_id"), OWNER))
        .to_owned();
    let product = Table::create()
        .table(Alias::new(PRODUCT))
        .col(id())
        .col(ColumnDef::new(Alias::new("shop_id")).integer().not_null())
        .foreign_key(&mut key(TO_SHOP_KEY, (PRODUCT, "shop_id"), SHOP))
        .to_owned();
    for create in [owner, shop, product] {
        db.execute(&create).await.expect("create a test table");
    }
    let index = Index::create()
        .name(SHOP_INDEX)
        .table(Alias::new(SHOP))
        .col(Alias::new("city"))
        .to_owned();
    db.execute(&index).await.expect("create the index");

    insert(db, OWNER, &["id"], &[1]).await.unwrap();
    insert(db, SHOP, &["id", "owner_id", "city"], &[1, 1, 7]).await.unwrap();
    insert(db, PRODUCT, &["id", "shop_id"], &[1, 1]).await.unwrap();
}

/// What the `up` of the generated migrations runs: the table, its index, its
/// key, then the key of the table pointing to it.
async fn rename_up(manager: &SchemaManager<'_>) {
    manager
        .rename_table(
            Table::rename()
                .table(Alias::new(SHOP), Alias::new(BAKERY))
                .to_owned(),
        )
        .await
        .expect("rename the table");
    manager
        .create_index(
            Index::create()
                .name(BAKERY_INDEX)
                .table(Alias::new(BAKERY))
                .col(Alias::new("city"))
                .to_owned(),
        )
        .await
        .expect("create the index under its new name");
    manager
        .drop_index(Index::drop().name(SHOP_INDEX).table(Alias::new(BAKERY)).to_owned())
        .await
        .expect("drop the index of the former name");
    rename_foreign_key(
        manager,
        BAKERY,
        SHOP_KEY,
        key(BAKERY_KEY, (BAKERY, "owner_id"), OWNER),
    )
    .await
    .expect("rename the table's own key");
    rename_foreign_key(
        manager,
        PRODUCT,
        TO_SHOP_KEY,
        key(TO_BAKERY_KEY, (PRODUCT, "shop_id"), BAKERY),
    )
    .await
    .expect("rename the key pointing to the table");
}

/// The `down`s, in the reverse order: the keys and the index take their
/// former names back while the table still has the new one.
async fn rename_down(manager: &SchemaManager<'_>) {
    rename_foreign_key(
        manager,
        PRODUCT,
        TO_BAKERY_KEY,
        key(TO_SHOP_KEY, (PRODUCT, "shop_id"), BAKERY),
    )
    .await
    .expect("give the pointing key its former name");
    rename_foreign_key(
        manager,
        BAKERY,
        BAKERY_KEY,
        key(SHOP_KEY, (BAKERY, "owner_id"), OWNER),
    )
    .await
    .expect("give the table's key its former name");
    manager
        .create_index(
            Index::create()
                .name(SHOP_INDEX)
                .table(Alias::new(BAKERY))
                .col(Alias::new("city"))
                .to_owned(),
        )
        .await
        .expect("create the index under its former name");
    manager
        .drop_index(Index::drop().name(BAKERY_INDEX).table(Alias::new(BAKERY)).to_owned())
        .await
        .expect("drop the index of the new name");
    manager
        .rename_table(
            Table::rename()
                .table(Alias::new(BAKERY), Alias::new(SHOP))
                .to_owned(),
        )
        .await
        .expect("rename the table back");
}

/// Both keys of the renamed table `shop` do their work: a row that points to
/// nothing is refused, a row that points to an existing one is taken.
async fn keys_hold(db: &DatabaseConnection, shop: &str, next_id: i32) {
    assert!(
        insert(db, shop, &["id", "owner_id", "city"], &[next_id, 999, 7]).await.is_err(),
        "`{shop}` took a row whose owner doesn't exist"
    );
    assert!(
        insert(db, PRODUCT, &["id", "shop_id"], &[next_id, 999]).await.is_err(),
        "`{PRODUCT}` took a row whose `{shop}` doesn't exist"
    );
    insert(db, PRODUCT, &["id", "shop_id"], &[next_id, 1])
        .await
        .expect("a row pointing to the renamed table");
}

async fn a_table_is_renamed_and_renamed_back(db: &DatabaseConnection) {
    three_tables(db).await;
    let manager = SchemaManager::new(db);
    let server = db.get_database_backend() != DbBackend::Sqlite;

    rename_up(&manager).await;

    assert!(manager.has_table(BAKERY).await.unwrap());
    assert!(!manager.has_table(SHOP).await.unwrap());
    assert_eq!(rows(db, BAKERY).await, 1, "the rows follow the table");
    assert!(manager.has_index(BAKERY, BAKERY_INDEX).await.unwrap());
    assert!(!manager.has_index(BAKERY, SHOP_INDEX).await.unwrap());
    keys_hold(db, BAKERY, 2).await;
    // A server knows its keys by name: the former names are gone.
    if server {
        assert!(drop_foreign_key(&manager, BAKERY, SHOP_KEY).await.is_err());
        assert!(drop_foreign_key(&manager, PRODUCT, TO_SHOP_KEY).await.is_err());
    }

    rename_down(&manager).await;

    assert!(manager.has_table(SHOP).await.unwrap());
    assert!(!manager.has_table(BAKERY).await.unwrap());
    assert_eq!(rows(db, SHOP).await, 1);
    assert!(manager.has_index(SHOP, SHOP_INDEX).await.unwrap());
    assert!(!manager.has_index(SHOP, BAKERY_INDEX).await.unwrap());
    keys_hold(db, SHOP, 3).await;
    // Back under their former names: they can be reached by them again.
    if server {
        assert!(drop_foreign_key(&manager, PRODUCT, TO_BAKERY_KEY).await.is_err());
        drop_foreign_key(&manager, PRODUCT, TO_SHOP_KEY)
            .await
            .expect("the pointing key has its former name");
        drop_foreign_key(&manager, SHOP, SHOP_KEY)
            .await
            .expect("the table's key has its former name");
    }
    drop_all(db).await;
}

#[tokio::test]
#[serial]
async fn a_table_is_renamed_and_renamed_back_on_postgres() {
    let Some(db) = db_postgres::connect().await else {
        return;
    };
    a_table_is_renamed_and_renamed_back(&db).await;
}

#[tokio::test]
#[serial]
async fn a_table_is_renamed_and_renamed_back_on_mysql() {
    let Some(db) = db_mysql::connect().await else {
        return;
    };
    a_table_is_renamed_and_renamed_back(&db).await;
}

#[tokio::test]
async fn a_table_is_renamed_and_renamed_back_on_sqlite() {
    let db = db::fresh_db().await;
    a_table_is_renamed_and_renamed_back(&db).await;
}
