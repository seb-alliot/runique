//! Tests — utils/reset_token
//! Couvre : generate, consume et peek (DB-backed).

use crate::helpers::db;
use crate::helpers::pk::pk;
use runique::utils::reset_token::{consume, generate, peek, resolve};
use std::time::Duration;

#[cfg(feature = "pk-uuid")]
const RESET_TOKENS_DDL: &str = "
    CREATE TABLE eihwaz_reset_tokens (
        id          INTEGER PRIMARY KEY AUTOINCREMENT,
        token_hash  TEXT NOT NULL UNIQUE,
        user_id     TEXT NOT NULL,
        expires_at  TEXT NOT NULL
    )
";

#[cfg(not(feature = "pk-uuid"))]
const RESET_TOKENS_DDL: &str = "
    CREATE TABLE eihwaz_reset_tokens (
        id          INTEGER PRIMARY KEY AUTOINCREMENT,
        token_hash  TEXT NOT NULL UNIQUE,
        user_id     INTEGER NOT NULL,
        expires_at  TEXT NOT NULL
    )
";

fn ttl() -> Duration {
    Duration::from_secs(3600)
}

// ═══════════════════════════════════════════════════════════════
// generate / peek / consume (DB-backed, hashed, single-use)
// ═══════════════════════════════════════════════════════════════

#[tokio::test]
async fn test_generate_peek_valid() {
    let conn = runique::db::ADb::from_connection(db::fresh_db_with_schema(RESET_TOKENS_DDL).await);
    let token = generate(&conn, pk(1), ttl()).await.unwrap();
    assert!(peek(&conn, &token).await);
}

#[tokio::test]
async fn test_resolve_returns_the_bound_user_without_consuming_the_token() {
    let conn = runique::db::ADb::from_connection(db::fresh_db_with_schema(RESET_TOKENS_DDL).await);
    let token = generate(&conn, pk(42), ttl()).await.unwrap();

    assert_eq!(resolve(&conn, &token).await, Some(pk(42)));
    assert!(peek(&conn, &token).await);
    assert_eq!(consume(&conn, &token).await, Some(pk(42)));
}

#[tokio::test]
async fn test_consume_returns_user_id() {
    let conn = runique::db::ADb::from_connection(db::fresh_db_with_schema(RESET_TOKENS_DDL).await);
    let token = generate(&conn, pk(42), ttl()).await.unwrap();
    assert_eq!(consume(&conn, &token).await, Some(pk(42)));
}

#[tokio::test]
async fn test_consume_single_use() {
    let conn = runique::db::ADb::from_connection(db::fresh_db_with_schema(RESET_TOKENS_DDL).await);
    let token = generate(&conn, pk(1), ttl()).await.unwrap();
    let _ = consume(&conn, &token).await;
    assert_eq!(consume(&conn, &token).await, None);
}

#[tokio::test]
async fn test_peek_after_consume_false() {
    let conn = runique::db::ADb::from_connection(db::fresh_db_with_schema(RESET_TOKENS_DDL).await);
    let token = generate(&conn, pk(1), ttl()).await.unwrap();
    let _ = consume(&conn, &token).await;
    assert!(!peek(&conn, &token).await);
}

#[tokio::test]
async fn test_consume_unknown_token() {
    let conn = runique::db::ADb::from_connection(db::fresh_db_with_schema(RESET_TOKENS_DDL).await);
    assert_eq!(consume(&conn, "non-existent-token-xyz").await, None);
}

#[tokio::test]
async fn test_peek_unknown_token() {
    let conn = runique::db::ADb::from_connection(db::fresh_db_with_schema(RESET_TOKENS_DDL).await);
    assert!(!peek(&conn, "non-existent-token-abc").await);
}

// Written from cargo-mutants survivors (2026-10-02): every test above passes
// with a `hash_token` that returns the same value for every token — then any
// made-up token would find the last row stored and reset that user's password.
#[tokio::test]
async fn test_a_live_token_never_lets_another_one_through() {
    let conn = runique::db::ADb::from_connection(db::fresh_db_with_schema(RESET_TOKENS_DDL).await);
    let alice = generate(&conn, pk(1), ttl()).await.unwrap();
    let bob = generate(&conn, pk(2), ttl()).await.unwrap();

    assert!(!peek(&conn, "made-up-token").await);
    assert_eq!(consume(&conn, "made-up-token").await, None);
    assert_eq!(consume(&conn, &bob).await, Some(pk(2)));
    assert_eq!(consume(&conn, &alice).await, Some(pk(1)));
}

#[tokio::test]
async fn test_only_the_token_hash_is_stored() {
    use runique::sea_orm::{ConnectionTrait, Statement};
    let conn = runique::db::ADb::from_connection(db::fresh_db_with_schema(RESET_TOKENS_DDL).await);
    let token = generate(&conn, pk(1), ttl()).await.unwrap();
    let stored: String = conn
        .query_one_raw(Statement::from_string(
            conn.get_database_backend(),
            "SELECT token_hash FROM eihwaz_reset_tokens",
        ))
        .await
        .unwrap()
        .unwrap()
        .try_get("", "token_hash")
        .unwrap();
    assert_ne!(stored, token);
    assert!(!stored.contains(&token));
    // base64url of a SHA-256 digest, without padding.
    assert_eq!(stored.len(), 43);
}
