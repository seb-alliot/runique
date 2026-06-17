//! Tests — utils/reset_token
//! Couvre : generate, consume, peek (DB-backed), encrypt_email, decrypt_email

use crate::helpers::db;
use runique::utils::reset_token::{consume, decrypt_email, encrypt_email, generate, peek};
use std::time::Duration;

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
    let conn = db::fresh_db_with_schema(RESET_TOKENS_DDL).await;
    let token = generate(&conn, 1, ttl()).await.unwrap();
    assert!(peek(&conn, &token).await);
}

#[tokio::test]
async fn test_consume_returns_user_id() {
    let conn = db::fresh_db_with_schema(RESET_TOKENS_DDL).await;
    let token = generate(&conn, 42, ttl()).await.unwrap();
    assert_eq!(consume(&conn, &token).await, Some(42));
}

#[tokio::test]
async fn test_consume_single_use() {
    let conn = db::fresh_db_with_schema(RESET_TOKENS_DDL).await;
    let token = generate(&conn, 1, ttl()).await.unwrap();
    let _ = consume(&conn, &token).await;
    assert_eq!(consume(&conn, &token).await, None);
}

#[tokio::test]
async fn test_peek_after_consume_false() {
    let conn = db::fresh_db_with_schema(RESET_TOKENS_DDL).await;
    let token = generate(&conn, 1, ttl()).await.unwrap();
    let _ = consume(&conn, &token).await;
    assert!(!peek(&conn, &token).await);
}

#[tokio::test]
async fn test_consume_unknown_token() {
    let conn = db::fresh_db_with_schema(RESET_TOKENS_DDL).await;
    assert_eq!(consume(&conn, "non-existent-token-xyz").await, None);
}

#[tokio::test]
async fn test_peek_unknown_token() {
    let conn = db::fresh_db_with_schema(RESET_TOKENS_DDL).await;
    assert!(!peek(&conn, "non-existent-token-abc").await);
}

// ═══════════════════════════════════════════════════════════════
// encrypt_email / decrypt_email
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_encrypt_decrypt_roundtrip() {
    let token = "test-token-roundtrip-abc";
    let email = "test@example.com";
    let encrypted = encrypt_email(token, email);
    let decrypted = decrypt_email(token, &encrypted);
    assert_eq!(decrypted, Some(email.to_string()));
}

#[test]
fn test_encrypt_decrypt_long_email() {
    let token = "token-long-email";
    let email = "very.long.email.address.for.testing@subdomain.example.com";
    let encrypted = encrypt_email(token, email);
    let decrypted = decrypt_email(token, &encrypted);
    assert_eq!(decrypted, Some(email.to_string()));
}

#[test]
fn test_decrypt_wrong_token_returns_none() {
    let encrypted = encrypt_email("correct-token", "test@example.com");
    let result = decrypt_email("wrong-token", &encrypted);
    assert_eq!(result, None);
}

#[test]
fn test_decrypt_invalid_base64_returns_none() {
    let result = decrypt_email("any-token", "not-valid-base64!!!!@#$");
    assert_eq!(result, None);
}

#[test]
fn test_decrypt_too_short_returns_none() {
    use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
    let short = URL_SAFE_NO_PAD.encode([0u8; 10]);
    let result = decrypt_email("any-token", &short);
    assert_eq!(result, None);
}

#[test]
fn test_encrypt_different_tokens_give_different_output() {
    let email = "same@example.com";
    let enc1 = encrypt_email("token-one", email);
    let enc2 = encrypt_email("token-two", email);
    assert_ne!(enc1, enc2);
}
