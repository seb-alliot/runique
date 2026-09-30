//! Whatever the handler does, nothing it wrote survives, and the next test
//! isn't held up or fed stale state.
use super::helpers::{Scratch, count, insert};
use crate::helpers::pk::pk;
use runique::auth::guard::{cache_permissions, evict_permissions, get_permissions};
use runique::auth::permissions::Groupe;
use runique::db::ADb;
use runique::runique_test::runique_test;
use runique::sea_orm::DbErr;
use serial_test::serial;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

#[tokio::test]
async fn writes_are_rolled_back() {
    let scratch = Scratch::new("rollback");
    let outside = scratch.outside().await;

    let outcome = runique_test::<ADb>(&scratch.sqlite_env(&[]), async |db| {
        insert(db, "kept_for_the_test_only").await?;
        assert_eq!(count(db).await, 1, "the row is visible inside the test");
        Ok(())
    })
    .await;

    assert!(outcome.is_ok(), "{outcome:?}");
    assert_eq!(count(&outside).await, 0);
}

#[tokio::test]
async fn an_error_still_rolls_back() {
    let scratch = Scratch::new("error");
    let outside = scratch.outside().await;

    let outcome = runique_test::<ADb>(&scratch.sqlite_env(&[]), async |db| {
        insert(db, "a").await?;
        Err(DbErr::Custom("fails on purpose".into()))
    })
    .await;

    assert!(outcome.is_err());
    assert_eq!(count(&outside).await, 0);
}

#[tokio::test]
async fn a_panic_is_caught_rolled_back_and_releases_the_lock() {
    let scratch = Scratch::new("panic");
    let outside = scratch.outside().await;
    let env = scratch.sqlite_env(&[]);

    let outcome = runique_test::<ADb>(&env, async |db| {
        insert(db, "a").await?;
        panic!("panics on purpose");
    })
    .await;
    assert!(outcome.is_err());
    assert_eq!(count(&outside).await, 0);

    // A lock left held by the panic would make this one wait forever.
    let next = tokio::time::timeout(
        Duration::from_secs(10),
        runique_test::<ADb>(&env, async |_| Ok(())),
    )
    .await;
    assert!(matches!(next, Ok(Ok(()))), "{next:?}");
}

#[tokio::test]
async fn a_nested_call_is_refused_instead_of_hanging() {
    let scratch = Scratch::new("nested");
    scratch.outside().await;
    let env = scratch.sqlite_env(&[]);

    let inner: Arc<Mutex<Option<bool>>> = Arc::default();
    let seen = inner.clone();
    let outer = tokio::time::timeout(
        Duration::from_secs(10),
        runique_test::<ADb>(&env, async |_| {
            let nested = runique_test::<ADb>(&env, async |_| Ok(())).await;
            *seen.lock().unwrap() = Some(nested.is_err());
            Ok(())
        }),
    )
    .await;

    assert!(
        matches!(outer, Ok(Ok(()))),
        "the outer test hung or failed: {outer:?}"
    );
    assert_eq!(
        *inner.lock().unwrap(),
        Some(true),
        "the nested call must be refused"
    );
}

#[tokio::test]
async fn a_handler_that_runs_too_long_is_stopped_and_rolled_back() {
    let scratch = Scratch::new("timeout");
    let outside = scratch.outside().await;
    let started = Instant::now();

    let outcome = runique_test::<ADb>(
        &scratch.sqlite_env(&["RUNIQUE_TEST_TIMEOUT=1"]),
        async |db| {
            insert(db, "a").await?;
            tokio::time::sleep(Duration::from_secs(30)).await;
            Ok(())
        },
    )
    .await;

    assert!(outcome.is_err());
    assert!(
        started.elapsed() < Duration::from_secs(10),
        "{:?}",
        started.elapsed()
    );
    assert_eq!(count(&outside).await, 0);
}

#[tokio::test]
async fn a_clone_dropped_soon_after_the_handler_is_waited_for() {
    let scratch = Scratch::new("clone_short");
    let outside = scratch.outside().await;

    let outcome = runique_test::<ADb>(&scratch.sqlite_env(&[]), async |db| {
        let db = db.clone();
        tokio::spawn(async move {
            tokio::time::sleep(Duration::from_millis(300)).await;
            insert(&db, "late").await.ok();
        });
        Ok(())
    })
    .await;

    assert!(outcome.is_ok(), "{outcome:?}");
    assert_eq!(
        count(&outside).await,
        0,
        "the late write must be rolled back too"
    );
}

#[tokio::test]
async fn a_clone_kept_too_long_is_reported_and_still_rolled_back() {
    let scratch = Scratch::new("clone_long");
    let outside = scratch.outside().await;
    let task: Arc<Mutex<Option<tokio::task::JoinHandle<()>>>> = Arc::default();
    let slot = task.clone();

    let outcome = runique_test::<ADb>(&scratch.sqlite_env(&[]), async |db| {
        let db = db.clone();
        *slot.lock().unwrap() = Some(tokio::spawn(async move {
            insert(&db, "late").await.ok();
            tokio::time::sleep(Duration::from_secs(5)).await;
        }));
        Ok(())
    })
    .await;

    assert!(
        outcome.is_err(),
        "a clone outliving the grace period must fail the test"
    );
    let handle = task.lock().unwrap().take().expect("spawned task");
    handle.await.expect("task");
    // Once the last clone is gone, the transaction is rolled back on drop.
    assert_eq!(count(&outside).await, 0);
}

fn group(id: i32) -> Vec<Groupe> {
    vec![Groupe {
        id,
        nom: format!("group {id}"),
        permissions: Vec::new(),
    }]
}

// `#[serial]`, like every test touching this global cache: another test's
// `clear_cache()` could otherwise wipe `kept` between the setup and the check.
#[tokio::test]
#[serial]
async fn the_permission_cache_is_put_back() {
    let scratch = Scratch::new("cache");
    scratch.outside().await;
    let (kept, added) = (pk(990_001), pk(990_002));
    cache_permissions(kept, group(1));

    let outcome = runique_test::<ADb>(&scratch.sqlite_env(&[]), async |_| {
        cache_permissions(added, group(2));
        evict_permissions(kept);
        Ok(())
    })
    .await;

    assert!(outcome.is_ok(), "{outcome:?}");
    assert!(
        get_permissions(added).is_none(),
        "an entry the test added must go"
    );
    let restored = get_permissions(kept).expect("an entry the test removed must come back");
    assert_eq!(restored.groupes[0].id, 1);
    evict_permissions(kept);
}
