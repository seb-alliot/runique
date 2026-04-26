//! Tests — middleware/session/session_db.rs : RuniqueSessionStore

use crate::helpers::db;
use runique::middleware::session::session_db::RuniqueSessionStore;
use std::sync::Arc;

const SESSIONS_DDL: &str = "
    CREATE TABLE eihwaz_sessions (
        id          INTEGER PRIMARY KEY AUTOINCREMENT,
        cookie_id   TEXT NOT NULL UNIQUE,
        user_id     INTEGER NOT NULL,
        session_id  TEXT NOT NULL,
        session_data TEXT,
        expires_at  TEXT NOT NULL
    )
";

fn future_expiry() -> chrono::NaiveDateTime {
    chrono::Utc::now()
        .naive_utc()
        .checked_add_signed(chrono::Duration::hours(24))
        .unwrap()
}

fn past_expiry() -> chrono::NaiveDateTime {
    chrono::Utc::now()
        .naive_utc()
        .checked_sub_signed(chrono::Duration::hours(1))
        .unwrap()
}

async fn make_store() -> RuniqueSessionStore {
    let db = db::fresh_db_with_schema(SESSIONS_DDL).await;
    RuniqueSessionStore::new(Arc::new(db))
}

// ═══════════════════════════════════════════════════════════════
// create / find_by_cookie_id
// ═══════════════════════════════════════════════════════════════

#[tokio::test]
async fn test_session_db_create_and_find() {
    let store = make_store().await;
    store
        .create("cookie-1", 1, "sess-1", future_expiry())
        .await
        .unwrap();

    let result = store.find_by_cookie_id("cookie-1").await.unwrap();
    assert!(result.is_some());
    let s = result.unwrap();
    assert_eq!(s.cookie_id, "cookie-1");
    assert_eq!(s.user_id, 1);
}

#[tokio::test]
async fn test_session_db_find_absent_returns_none() {
    let store = make_store().await;
    let result = store.find_by_cookie_id("nonexistent").await.unwrap();
    assert!(result.is_none());
}

#[tokio::test]
async fn test_session_db_find_expired_returns_none() {
    let store = make_store().await;
    store
        .create("cookie-expired", 1, "sess-exp", past_expiry())
        .await
        .unwrap();

    let result = store.find_by_cookie_id("cookie-expired").await.unwrap();
    assert!(result.is_none());
}

// ═══════════════════════════════════════════════════════════════
// delete
// ═══════════════════════════════════════════════════════════════

#[tokio::test]
async fn test_session_db_delete() {
    let store = make_store().await;
    store
        .create("cookie-del", 1, "sess-del", future_expiry())
        .await
        .unwrap();
    store.delete("cookie-del").await.unwrap();

    let result = store.find_by_cookie_id("cookie-del").await.unwrap();
    assert!(result.is_none());
}

#[tokio::test]
async fn test_session_db_delete_nonexistent_ok() {
    let store = make_store().await;
    // Suppression d'une session inexistante → pas d'erreur
    assert!(store.delete("ghost").await.is_ok());
}

// ═══════════════════════════════════════════════════════════════
// invalidate_other_sessions
// ═══════════════════════════════════════════════════════════════

#[tokio::test]
async fn test_session_db_invalidate_other_sessions() {
    let store = make_store().await;
    store
        .create("cookie-a", 5, "sess-a", future_expiry())
        .await
        .unwrap();
    store
        .create("cookie-b", 5, "sess-b", future_expiry())
        .await
        .unwrap();
    store
        .create("cookie-c", 5, "sess-c", future_expiry())
        .await
        .unwrap();

    // Garde cookie-b, supprime les autres de l'user 5
    store
        .invalidate_other_sessions(5, "cookie-b")
        .await
        .unwrap();

    assert!(store.find_by_cookie_id("cookie-a").await.unwrap().is_none());
    assert!(store.find_by_cookie_id("cookie-b").await.unwrap().is_some());
    assert!(store.find_by_cookie_id("cookie-c").await.unwrap().is_none());
}

// ═══════════════════════════════════════════════════════════════
// invalidate_all
// ═══════════════════════════════════════════════════════════════

#[tokio::test]
async fn test_session_db_invalidate_all() {
    let store = make_store().await;
    store
        .create("cookie-x", 7, "sess-x", future_expiry())
        .await
        .unwrap();
    store
        .create("cookie-y", 7, "sess-y", future_expiry())
        .await
        .unwrap();

    store.invalidate_all(7).await.unwrap();

    assert!(store.find_by_cookie_id("cookie-x").await.unwrap().is_none());
    assert!(store.find_by_cookie_id("cookie-y").await.unwrap().is_none());
}

// ═══════════════════════════════════════════════════════════════
// update_session_data
// ═══════════════════════════════════════════════════════════════

#[tokio::test]
async fn test_session_db_update_session_data() {
    let store = make_store().await;
    store
        .create("cookie-data", 2, "sess-data", future_expiry())
        .await
        .unwrap();

    store
        .update_session_data("cookie-data", Some("{\"key\":\"value\"}".to_string()))
        .await
        .unwrap();

    let s = store
        .find_by_cookie_id("cookie-data")
        .await
        .unwrap()
        .unwrap();
    assert_eq!(s.session_data, Some("{\"key\":\"value\"}".to_string()));
}

#[tokio::test]
async fn test_session_db_update_session_data_none() {
    let store = make_store().await;
    store
        .create("cookie-null", 2, "sess-null", future_expiry())
        .await
        .unwrap();
    assert!(store.update_session_data("cookie-null", None).await.is_ok());
}

// ═══════════════════════════════════════════════════════════════
// find_by_user
// ═══════════════════════════════════════════════════════════════

#[tokio::test]
async fn test_session_db_find_by_user() {
    let store = make_store().await;
    store
        .create("cookie-u1", 10, "sess-u1", future_expiry())
        .await
        .unwrap();
    store
        .create("cookie-u2", 10, "sess-u2", future_expiry())
        .await
        .unwrap();
    store
        .create("cookie-other", 99, "sess-other", future_expiry())
        .await
        .unwrap();

    let sessions = store.find_by_user(10).await.unwrap();
    assert_eq!(sessions.len(), 2);
}

#[tokio::test]
async fn test_session_db_find_by_user_excludes_expired() {
    let store = make_store().await;
    store
        .create("cookie-active", 11, "sess-act", future_expiry())
        .await
        .unwrap();
    store
        .create("cookie-old", 11, "sess-old", past_expiry())
        .await
        .unwrap();

    let sessions = store.find_by_user(11).await.unwrap();
    assert_eq!(sessions.len(), 1);
    assert_eq!(sessions[0].cookie_id, "cookie-active");
}

// ═══════════════════════════════════════════════════════════════
// spawn_cleanup
// ═══════════════════════════════════════════════════════════════

#[tokio::test]
async fn test_session_db_spawn_cleanup_no_panic() {
    let store = make_store().await;
    store.spawn_cleanup(tokio::time::Duration::from_secs(60));
    // Tâche tokio en arrière-plan — on vérifie juste que ça ne panique pas
}
