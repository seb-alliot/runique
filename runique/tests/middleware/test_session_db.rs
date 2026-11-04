//! Tests — middleware/session/session_db.rs : RuniqueSessionStore

use crate::helpers::db;
use crate::helpers::pk::pk;
use runique::middleware::session::session_db::RuniqueSessionStore;

#[cfg(feature = "pk-uuid")]
pub(crate) const SESSIONS_DDL: &str = "
    CREATE TABLE eihwaz_sessions (
        id          INTEGER PRIMARY KEY AUTOINCREMENT,
        cookie_id   TEXT NOT NULL UNIQUE,
        user_id     TEXT NOT NULL,
        session_id  TEXT NOT NULL,
        session_data TEXT,
        expires_at  TEXT NOT NULL
    )
";

#[cfg(not(feature = "pk-uuid"))]
pub(crate) const SESSIONS_DDL: &str = "
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
    RuniqueSessionStore::new(
        runique::db::ADb::from_connection(db),
        crate::helpers::server::TEST_SECRET,
    )
}

// ═══════════════════════════════════════════════════════════════
// create / find_by_cookie_id
// ═══════════════════════════════════════════════════════════════

#[tokio::test]
async fn test_session_db_create_and_find() {
    let store = make_store().await;
    store
        .create("cookie-1", pk(1), "sess-1", future_expiry())
        .await
        .unwrap();

    let result = store.find_by_cookie_id("cookie-1").await.unwrap();
    assert!(result.is_some());
    let s = result.unwrap();
    assert_eq!(s.cookie_id, "cookie-1");
    assert_eq!(s.user_id, pk(1));
}

#[tokio::test]
async fn test_session_db_does_not_store_the_raw_cookie_id() {
    use runique::sea_orm::{ConnectionTrait, Statement};

    let conn = db::fresh_db_with_schema(SESSIONS_DDL).await;
    let store = RuniqueSessionStore::new(
        runique::db::ADb::from_connection(conn.clone()),
        crate::helpers::server::TEST_SECRET,
    );
    let cookie_id = "sensitive-session-cookie";
    store
        .create(cookie_id, pk(1), "sess-1", future_expiry())
        .await
        .unwrap();

    let stored: String = conn
        .query_one_raw(Statement::from_string(
            conn.get_database_backend(),
            "SELECT cookie_id FROM eihwaz_sessions",
        ))
        .await
        .unwrap()
        .unwrap()
        .try_get("", "cookie_id")
        .unwrap();

    assert_ne!(
        stored, cookie_id,
        "a read-only database leak must not reveal a replayable session cookie"
    );
    assert!(store.find_by_cookie_id(cookie_id).await.unwrap().is_some());
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
        .create("cookie-expired", pk(1), "sess-exp", past_expiry())
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
        .create("cookie-del", pk(1), "sess-del", future_expiry())
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
        .create("cookie-a", pk(5), "sess-a", future_expiry())
        .await
        .unwrap();
    store
        .create("cookie-b", pk(5), "sess-b", future_expiry())
        .await
        .unwrap();
    store
        .create("cookie-c", pk(5), "sess-c", future_expiry())
        .await
        .unwrap();

    // Garde cookie-b, supprime les autres de l'user 5
    store
        .invalidate_other_sessions(pk(5), "cookie-b")
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
        .create("cookie-x", pk(7), "sess-x", future_expiry())
        .await
        .unwrap();
    store
        .create("cookie-y", pk(7), "sess-y", future_expiry())
        .await
        .unwrap();

    store.invalidate_all(pk(7)).await.unwrap();

    assert!(store.find_by_cookie_id("cookie-x").await.unwrap().is_none());
    assert!(store.find_by_cookie_id("cookie-y").await.unwrap().is_none());
}

// ═══════════════════════════════════════════════════════════════
// upsert_session — single full-snapshot write path (data + expiry)
// ═══════════════════════════════════════════════════════════════

#[tokio::test]
async fn test_session_db_upsert_inserts_when_absent() {
    let store = make_store().await;
    store
        .upsert_session(
            "cookie-up",
            pk(3),
            future_expiry(),
            Some("{\"key\":\"value\"}".to_string()),
        )
        .await
        .unwrap();

    let s = store.find_by_cookie_id("cookie-up").await.unwrap().unwrap();
    assert_eq!(s.user_id, pk(3));
    assert_eq!(
        store.open_session_data(&s),
        Some("{\"key\":\"value\"}".to_string())
    );
}

#[tokio::test]
async fn test_session_db_upsert_updates_data_on_existing() {
    let store = make_store().await;
    store
        .create("cookie-ud", pk(2), "sess-ud", future_expiry())
        .await
        .unwrap();

    store
        .upsert_session(
            "cookie-ud",
            pk(2),
            future_expiry(),
            Some("{\"v\":2}".to_string()),
        )
        .await
        .unwrap();

    let s = store.find_by_cookie_id("cookie-ud").await.unwrap().unwrap();
    assert_eq!(store.open_session_data(&s), Some("{\"v\":2}".to_string()));
}

// ═══════════════════════════════════════════════════════════════
// session_data encrypted at rest
// ═══════════════════════════════════════════════════════════════

const SECRET_DATA: &str = "{\"csrf\":\"top-secret\"}";

/// What the table really holds, without going through the store.
async fn stored_data(conn: &runique::sea_orm::DatabaseConnection) -> String {
    use runique::sea_orm::{ConnectionTrait, Statement};
    conn.query_one_raw(Statement::from_string(
        conn.get_database_backend(),
        "SELECT session_data FROM eihwaz_sessions",
    ))
    .await
    .unwrap()
    .unwrap()
    .try_get("", "session_data")
    .unwrap()
}

#[tokio::test]
async fn both_writes_store_the_session_data_encrypted() {
    let conn = db::fresh_db_with_schema(SESSIONS_DDL).await;
    let store = RuniqueSessionStore::new(
        runique::db::ADb::from_connection(conn.clone()),
        crate::helpers::server::TEST_SECRET,
    );

    store
        .upsert_session("c", pk(1), future_expiry(), Some(SECRET_DATA.to_string()))
        .await
        .unwrap();
    let after_upsert = stored_data(&conn).await;
    assert!(after_upsert.starts_with("v1:"), "{after_upsert}");
    assert!(!after_upsert.contains("top-secret"));

    assert!(
        store
            .update_session("c", pk(1), future_expiry(), Some(SECRET_DATA.to_string()))
            .await
            .unwrap()
    );
    let after_update = stored_data(&conn).await;
    assert!(after_update.starts_with("v1:"), "{after_update}");
    assert!(!after_update.contains("top-secret"));
    assert_ne!(
        after_upsert, after_update,
        "the same data encrypts differently on each write"
    );

    let row = store.find_by_cookie_id("c").await.unwrap().unwrap();
    assert_eq!(store.open_session_data(&row).as_deref(), Some(SECRET_DATA));
}

#[tokio::test]
async fn a_session_without_data_stays_without_data() {
    let store = make_store().await;
    store
        .upsert_session("c", pk(1), future_expiry(), None)
        .await
        .unwrap();

    let row = store.find_by_cookie_id("c").await.unwrap().unwrap();
    assert_eq!(row.session_data, None);
    assert_eq!(store.open_session_data(&row), None);
}

#[tokio::test]
async fn another_secret_key_cannot_read_the_session_data() {
    let conn = db::fresh_db_with_schema(SESSIONS_DDL).await;
    let store = RuniqueSessionStore::new(
        runique::db::ADb::from_connection(conn.clone()),
        crate::helpers::server::TEST_SECRET,
    );
    let other = RuniqueSessionStore::new(
        runique::db::ADb::from_connection(conn),
        "another_secret_key_of_at_least_32_chars",
    );
    store
        .upsert_session("c", pk(1), future_expiry(), Some(SECRET_DATA.to_string()))
        .await
        .unwrap();

    let row = other.find_by_cookie_id("c").await.unwrap().unwrap();
    assert_eq!(other.open_session_data(&row), None);
}

#[tokio::test]
async fn a_payload_altered_copied_or_in_clear_is_refused() {
    let store = make_store().await;
    store
        .upsert_session("c", pk(1), future_expiry(), Some(SECRET_DATA.to_string()))
        .await
        .unwrap();
    let row = store.find_by_cookie_id("c").await.unwrap().unwrap();
    let sealed = row.session_data.clone().unwrap();
    assert!(
        store.open_session_data(&row).is_some(),
        "the row itself opens"
    );

    let with_data = |data: &str| {
        let mut changed = row.clone();
        changed.session_data = Some(data.to_string());
        store.open_session_data(&changed)
    };

    let last = sealed.len() - 4;
    let flipped = if &sealed[last..last + 1] == "A" {
        "B"
    } else {
        "A"
    };
    let altered = format!("{}{}{}", &sealed[..last], flipped, &sealed[last + 1..]);
    assert_eq!(with_data(&altered), None, "altered");

    assert_eq!(with_data(SECRET_DATA), None, "in clear, from before 4.0");
    assert_eq!(with_data("v1:"), None, "nothing after the format");
    assert_eq!(with_data("v1:AAAA"), None, "shorter than a nonce");
    assert_eq!(with_data("v1:not base64!"), None, "not base64");
    assert_eq!(
        with_data(sealed.trim_start_matches("v1:")),
        None,
        "no format"
    );

    let mut copied = row.clone();
    copied.cookie_id = "another-cookie".to_string();
    assert_eq!(
        store.open_session_data(&copied),
        None,
        "copied on another row"
    );
}

#[tokio::test]
async fn test_session_db_upsert_refreshes_expiry_not_frozen() {
    // Regression (2026-06-15): a second write must move expires_at forward instead of
    // leaving the row frozen at the first (short) expiry — the bug that logged active
    // users out after a server restart because the DB backup looked expired.
    let store = make_store().await;
    let soon = chrono::Utc::now()
        .naive_utc()
        .checked_add_signed(chrono::Duration::minutes(5))
        .unwrap();
    store
        .upsert_session("cookie-exp", pk(4), soon, Some("{}".to_string()))
        .await
        .unwrap();
    store
        .upsert_session("cookie-exp", pk(4), future_expiry(), Some("{}".to_string()))
        .await
        .unwrap();

    let s = store
        .find_by_cookie_id("cookie-exp")
        .await
        .unwrap()
        .unwrap();
    assert!(
        s.expires_at > soon,
        "second upsert must refresh expires_at, got {}",
        s.expires_at
    );
}

// ═══════════════════════════════════════════════════════════════
// spawn_cleanup
// ═══════════════════════════════════════════════════════════════

#[tokio::test]
async fn test_session_db_spawn_cleanup_actually_purges_expired() {
    // `find_by_cookie_id` ne renvoie que les sessions ACTIVES (filtre `expires_at
    // > now`), donc inutilisable pour vérifier la présence d'une session déjà
    // expirée avant le cleanup — on compte la ligne brute dans la table à la place.
    let conn = crate::helpers::db::fresh_db_with_schema(SESSIONS_DDL).await;
    let store = RuniqueSessionStore::new(
        runique::db::ADb::from_connection(conn.clone()),
        crate::helpers::server::TEST_SECRET,
    );
    store
        .create("cookie-expired", pk(1), "sess-expired", past_expiry())
        .await
        .unwrap();
    crate::helpers::db::assert_count(&conn, "eihwaz_sessions", 1).await;

    // Intervalle court pour observer un vrai cycle de purge, pas juste l'absence de panic.
    store.spawn_cleanup(tokio::time::Duration::from_millis(20));
    tokio::time::sleep(tokio::time::Duration::from_millis(80)).await;

    crate::helpers::db::assert_count(&conn, "eihwaz_sessions", 0).await;
}
