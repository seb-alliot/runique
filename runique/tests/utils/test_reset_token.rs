//! Tests — utils/reset_token
//! Couvre : generate, consume, peek, encrypt_email, decrypt_email

use runique::utils::reset_token::{consume, decrypt_email, encrypt_email, generate, peek};

// ═══════════════════════════════════════════════════════════════
// generate / peek / consume
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_generate_peek_valid() {
    let token = generate("alice@example.com");
    assert!(peek(&token));
}

#[test]
fn test_consume_returns_email() {
    let token = generate("bob@example.com");
    let result = consume(&token);
    assert_eq!(result, Some("bob@example.com".to_string()));
}

#[test]
fn test_consume_single_use() {
    let token = generate("carol@example.com");
    let _ = consume(&token);
    assert_eq!(consume(&token), None);
}

#[test]
fn test_peek_after_consume_false() {
    let token = generate("dave@example.com");
    let _ = consume(&token);
    assert!(!peek(&token));
}

#[test]
fn test_consume_unknown_token() {
    assert_eq!(consume("non-existent-token-xyz-abc-123"), None);
}

#[test]
fn test_peek_unknown_token() {
    assert!(!peek("non-existent-token-xyz-abc-456"));
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
    let short = URL_SAFE_NO_PAD.encode(&[0u8; 10]);
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
