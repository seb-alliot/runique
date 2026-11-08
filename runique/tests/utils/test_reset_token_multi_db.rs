//! Tests — `reset_token::generate` on Postgres and MariaDB (Docker), against the
//! table the real migration creates: its `UNIQUE (user_id)` is what makes a
//! second request overwrite the first link. Postgres refuses `ON CONFLICT
//! (user_id)` without that index; MariaDB's `ON DUPLICATE KEY UPDATE` would
//! silently insert a second row. SQLite is covered by `test_reset_token.rs`.
//!
//! Needs `docker compose up -d` and `DATABASE_URL_PG` / `DATABASE_URL_MARIADB`
//! in `.env.test`; without them each test returns at once.

#[cfg(feature = "pk-uuid")]
use crate::helpers::pk::pk;
use crate::helpers::{db_mariadb, db_postgres};
use runique::auth::migrations_table::create_eihwaz_reset_tokens_table;
use runique::auth::user;
use runique::sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ConnectionTrait, DatabaseConnection, DbBackend, Schema,
    Statement,
};
use runique::utils::reset_token::{generate, peek};
use serial_test::serial;
use std::time::Duration;

/// Drops then recreates `eihwaz_reset_tokens` and `eihwaz_users`. The shared
/// Docker databases hold other tables pointing at `eihwaz_users`: Postgres
/// needs CASCADE, MariaDB needs its FK checks off, on one pinned connection.
async fn recreate_tables(db: &DatabaseConnection) {
    let backend = db.get_database_backend();
    match backend {
        DbBackend::Postgres => {
            for table in ["eihwaz_reset_tokens", "eihwaz_users"] {
                db.execute_unprepared(&format!("DROP TABLE IF EXISTS {table} CASCADE"))
                    .await
                    .expect("drop");
            }
        }
        DbBackend::MySql => {
            // `eihwaz_reset_tokens` is among the tables pointing at `eihwaz_users`.
            crate::helpers::db_mariadb::drop_users_and_dependents(db).await;
        }
        other => unimplemented!("engine not covered here: {other:?}"),
    }
    db.execute(&Schema::new(backend).create_table_from_entity(user::Entity))
        .await
        .expect("create eihwaz_users");
    db.execute(&create_eihwaz_reset_tokens_table())
        .await
        .expect("create eihwaz_reset_tokens");
}

async fn insert_alice(db: &DatabaseConnection) -> runique::utils::pk::Pk {
    #[allow(unused_mut)]
    let mut am = user::ActiveModel {
        username: Set("alice".to_string()),
        email: Set("alice@example.com".to_string()),
        password: Set("hash123".to_string()),
        is_active: Set(true),
        is_dirigeant: Set(false),
        is_admin: Set(false),
        is_superuser: Set(false),
        created_at: Set(None),
        updated_at: Set(None),
        ..Default::default()
    };
    #[cfg(feature = "pk-uuid")]
    {
        am.id = Set(pk(1));
    }
    am.insert(db).await.expect("insert alice").id
}

async fn rows(db: &DatabaseConnection) -> i64 {
    db.query_one_raw(Statement::from_string(
        db.get_database_backend(),
        "SELECT COUNT(*) AS n FROM eihwaz_reset_tokens",
    ))
    .await
    .unwrap()
    .unwrap()
    .try_get("", "n")
    .unwrap()
}

/// A second link for the same account replaces the first: one row, only the
/// newer token works.
async fn a_new_token_replaces_the_previous_one(db: DatabaseConnection) {
    recreate_tables(&db).await;
    let alice = insert_alice(&db).await;
    let adb = runique::db::ADb::from_connection(db.clone());
    let ttl = Duration::from_secs(600);

    let first = generate(&adb, alice, ttl).await.expect("first link");
    let second = generate(&adb, alice, ttl)
        .await
        .expect("second link, an update");

    assert!(!peek(&adb, &first).await, "the older link is dead");
    assert!(peek(&adb, &second).await, "the newer link works");
    assert_eq!(rows(&db).await, 1);
}

#[tokio::test]
#[serial]
async fn test_a_new_token_replaces_the_previous_one_pg() {
    let Some(db) = db_postgres::connect().await else {
        return;
    };
    a_new_token_replaces_the_previous_one(db).await;
}

#[tokio::test]
#[serial]
async fn test_a_new_token_replaces_the_previous_one_mariadb() {
    let Some(db) = db_mariadb::connect().await else {
        return;
    };
    a_new_token_replaces_the_previous_one(db).await;
}
