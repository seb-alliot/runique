//! Tests — `RuniqueQueryBuilder::order_by_random()` on the 3 supported engines
//! (SQLite, Postgres, MariaDB via Docker).
//!
//! This is a regression test for the 2026-09-20 fix: `order_by_random()` used
//! to emit a hardcoded `RANDOM()`, which SQLite/Postgres accept but MySQL/
//! MariaDB reject (they require `RAND()`). A syntax read isn't proof — this
//! actually executes the query against each live engine and checks two real
//! properties of the result, not just "it didn't error":
//!   1. completeness — every seeded row comes back, none lost/duplicated
//!   2. actual randomization — repeated runs don't all return the same order
//!      (if `order_by_random()` silently degraded to a no-op/default order,
//!      this would catch it; a plain "not empty" check would not)
//!
//! Postgres/MariaDB need `docker compose up -d` and `DATABASE_URL_PG`/
//! `DATABASE_URL_MARIADB` in `.env.test`. Absent → each test returns
//! immediately (skip), same convention as `test_user_model_multi_db.rs`.

#[cfg(feature = "pk-uuid")]
use crate::helpers::pk::pk;
use crate::helpers::{db, db_mariadb, db_postgres};
use runique::auth::user;
use runique::macros::bdd::query::RuniqueQueryBuilder;
use runique::sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ConnectionTrait, DatabaseConnection, DbBackend,
    EntityTrait, Schema, TransactionTrait,
};
use runique::utils::ADb;
use serial_test::serial;
use std::collections::HashSet;

// Same multi-engine DROP/CREATE dance as `test_user_model_multi_db.rs` — see
// that file's comment for why Postgres needs CASCADE and MariaDB needs the
// FK-checks toggle.
//
// MariaDB's `SET`/`DROP`/`SET` sequence runs inside a transaction (`db.begin()`)
// specifically to pin all three statements to the *same* pooled connection.
// `FOREIGN_KEY_CHECKS` is a session variable — `DatabaseConnection` is backed
// by a connection pool, so three separate `execute_unprepared()` calls on `db`
// directly are not guaranteed to land on the same physical connection. If the
// `DROP` lands on a different one than the `SET`, the constraint is still
// enforced there and the drop fails with error 1451. A `DatabaseTransaction`
// owns one connection for its whole lifetime, so routing all three through it
// closes that gap. (MariaDB implicitly commits DDL anyway, so this isn't about
// rollback semantics — only about connection pinning.)
async fn recreate_users_table(db: &DatabaseConnection) {
    let backend = db.get_database_backend();
    match backend {
        DbBackend::Postgres => {
            db.execute_unprepared("DROP TABLE IF EXISTS eihwaz_users CASCADE")
                .await
                .expect("drop eihwaz_users");
        }
        DbBackend::MySql => {
            let txn = db.begin().await.expect("begin txn for FK-safe drop");
            txn.execute_unprepared("SET FOREIGN_KEY_CHECKS=0")
                .await
                .expect("disable FK checks");
            txn.execute_unprepared("DROP TABLE IF EXISTS eihwaz_users")
                .await
                .expect("drop eihwaz_users");
            txn.execute_unprepared("SET FOREIGN_KEY_CHECKS=1")
                .await
                .expect("re-enable FK checks");
            txn.commit().await.expect("commit FK-safe drop txn");
        }
        DbBackend::Sqlite => {
            db.execute_unprepared("DROP TABLE IF EXISTS eihwaz_users")
                .await
                .expect("drop eihwaz_users");
        }
        other => unimplemented!("engine not covered by this test: {other:?}"),
    }
    let schema = Schema::new(backend);
    let stmt = schema.create_table_from_entity(user::Entity);
    db.execute(&stmt).await.expect("create eihwaz_users");
}

// 20 rows: enough that the odds of 5 independent random permutations
// coincidentally landing on the exact same order (1/20! per repeat) are
// effectively zero — a real signal, not a coin flip that could flake.
const SEED_COUNT: u32 = 20;
const RUNS: usize = 5;

async fn seed_users(db: &DatabaseConnection) -> HashSet<runique::utils::config::Pk> {
    let mut ids = HashSet::new();
    for i in 0..SEED_COUNT {
        #[allow(unused_mut)]
        let mut am = user::ActiveModel {
            username: Set(format!("user{i}")),
            email: Set(format!("user{i}@example.com")),
            password: Set("hash".to_string()),
            is_active: Set(true),
            is_staff: Set(false),
            is_superuser: Set(false),
            created_at: Set(None),
            updated_at: Set(None),
            ..Default::default()
        };
        // Uuid PKs are never DB auto-increment — generated application-side.
        #[cfg(feature = "pk-uuid")]
        {
            am.id = Set(pk(i + 1));
        }
        let model = am.insert(db).await.expect("insert seed user");
        ids.insert(model.id);
    }
    ids
}

/// The actual regression check — not just "did it error", but does the
/// *result* hold up: every seeded row comes back exactly once (completeness),
/// on every one of `RUNS` executions, and at least two of those executions
/// return the rows in a different order (proof `order_by_random()` is really
/// randomizing, not silently falling back to a stable default order — which
/// a mere "not empty"/"no SQL error" check would miss entirely).
async fn assert_order_by_random_result(
    db: &ADb,
    expected_ids: &HashSet<runique::utils::config::Pk>,
) {
    let mut orderings = Vec::with_capacity(RUNS);

    for run in 0..RUNS {
        let rows = RuniqueQueryBuilder::new(user::Entity::find())
            .order_by_random(db)
            .all(db)
            .await
            .expect("order_by_random() must execute without a SQL error on this engine");

        let returned_ids: HashSet<_> = rows.iter().map(|m| m.id).collect();
        assert_eq!(
            &returned_ids, expected_ids,
            "run {run}: returned row set must exactly match the seeded rows (no loss/duplication)"
        );

        orderings.push(rows.into_iter().map(|m| m.id).collect::<Vec<_>>());
    }

    let all_identical = orderings.windows(2).all(|w| w[0] == w[1]);
    assert!(
        !all_identical,
        "all {RUNS} runs returned the exact same order — order_by_random() is not \
         actually randomizing (this is what a plain \"query didn't error\" check would miss)"
    );
}

// ═══════════════════════════════════════════════════════════════
// SQLite
// ═══════════════════════════════════════════════════════════════

#[tokio::test]
async fn test_order_by_random_sqlite() {
    let db = db::fresh_db().await;
    recreate_users_table(&db).await;
    let ids = seed_users(&db).await;
    assert_order_by_random_result(&std::sync::Arc::new(db), &ids).await;
}

// ═══════════════════════════════════════════════════════════════
// PostgreSQL
// ═══════════════════════════════════════════════════════════════

#[tokio::test]
#[serial]
async fn test_order_by_random_pg() {
    let Some(db) = db_postgres::connect().await else {
        return;
    };
    recreate_users_table(&db).await;
    let ids = seed_users(&db).await;
    assert_order_by_random_result(&std::sync::Arc::new(db.clone()), &ids).await;

    db_postgres::exec(&db, "DROP TABLE IF EXISTS eihwaz_users CASCADE").await;
}

// ═══════════════════════════════════════════════════════════════
// MariaDB
// ═══════════════════════════════════════════════════════════════

#[tokio::test]
#[serial]
async fn test_order_by_random_mariadb() {
    let Some(db) = db_mariadb::connect().await else {
        return;
    };
    recreate_users_table(&db).await;
    let ids = seed_users(&db).await;
    assert_order_by_random_result(&std::sync::Arc::new(db.clone()), &ids).await;

    let txn = db.begin().await.expect("begin txn for FK-safe cleanup");
    txn.execute_unprepared("SET FOREIGN_KEY_CHECKS=0")
        .await
        .expect("disable FK checks");
    txn.execute_unprepared("DROP TABLE IF EXISTS eihwaz_users")
        .await
        .expect("drop eihwaz_users");
    txn.execute_unprepared("SET FOREIGN_KEY_CHECKS=1")
        .await
        .expect("re-enable FK checks");
    txn.commit().await.expect("commit FK-safe cleanup txn");
}
