//! Password reset tokens — HMAC-SHA256 generation, configurable TTL, memory storage.
use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use hmac::{Hmac, Mac};
use sha2::{Digest, Sha256};
use std::{
    collections::HashMap,
    sync::{LazyLock, Mutex},
    time::{Duration, Instant},
};
use subtle::ConstantTimeEq;
use uuid::Uuid;

type HmacSha256 = Hmac<Sha256>;

#[derive(Debug, Clone)]
struct ResetEntry {
    email: String,
    expires_at: Instant,
}

static TOKENS: LazyLock<Mutex<HashMap<String, ResetEntry>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

const TOKEN_TTL: Duration = Duration::from_secs(3600);

/// Generates a reset token linked to an email (valid 1h, single use).
pub fn generate(email: &str) -> String {
    let token = Uuid::new_v4().to_string();
    let mut store = TOKENS.lock().unwrap();
    let now = Instant::now();
    store.retain(|_, v| v.expires_at > now);
    store.insert(
        token.clone(),
        ResetEntry {
            email: email.to_string(),
            expires_at: now.checked_add(TOKEN_TTL).unwrap_or(now),
        },
    );
    token
}

/// Consumes a token (single use) → returns associated email if valid.
pub fn consume(token: &str) -> Option<String> {
    let mut store = TOKENS.lock().unwrap();
    let now = Instant::now();
    if let Some(entry) = store.remove(token)
        && entry.expires_at > now
    {
        return Some(entry.email);
    }
    None
}

/// Encrypts email with token as key (CTR-SHA256 + HMAC-SHA256).
/// Result: base64url, safe in a URL.
///
/// Construction: `enc_key` = SHA256(token || ":enc"), keystream[i] = `SHA256(enc_key` || `i_le32`)
/// Authentication: HMAC-SHA256(token, ciphertext), tag truncated to 16 bytes as prefix.
#[must_use]
pub fn encrypt_email(token: &str, email: &str) -> String {
    let email_bytes = email.as_bytes();

    // Encryption key derived from token — unique per token (UUID4)
    let enc_key: [u8; 32] = {
        let mut h = Sha256::new();
        h.update(token.as_bytes());
        h.update(b":enc");
        h.finalize().into()
    };

    // CTR encryption: keystream[i] = SHA256(enc_key || counter_le32)
    let mut ciphertext = Vec::with_capacity(email_bytes.len());
    for (block_idx, chunk) in email_bytes.chunks(32).enumerate() {
        let mut block_input = [0u8; 36];
        block_input[..32].copy_from_slice(&enc_key);
        block_input[32..].copy_from_slice(&u32::try_from(block_idx).expect("REASON").to_le_bytes());
        let keystream: [u8; 32] = Sha256::digest(block_input).into();
        for (b, k) in chunk.iter().zip(keystream.iter()) {
            ciphertext.push(b ^ k);
        }
    }

    // Authentication tag: HMAC-SHA256(token, ciphertext), truncated to 16 bytes
    let mut mac =
        HmacSha256::new_from_slice(token.as_bytes()).expect("HMAC-SHA256 accepts any key length");
    mac.update(&ciphertext);
    let tag = mac.finalize().into_bytes();

    // Output: tag[0..16] || ciphertext (tag as prefix for fast rejection)
    let mut output = Vec::with_capacity(16usize.saturating_add(ciphertext.len()));
    output.extend_from_slice(&tag[..16]);
    output.extend_from_slice(&ciphertext);

    URL_SAFE_NO_PAD.encode(&output)
}

/// Decrypts email from a base64url value produced by `encrypt_email`.
/// Returns `None` if decoding, authentication or UTF-8 fails.
#[must_use]
pub fn decrypt_email(token: &str, encoded: &str) -> Option<String> {
    let data = URL_SAFE_NO_PAD.decode(encoded).ok()?;
    // Minimum: 16 bytes of tag + at least 1 byte of ciphertext
    if data.len() < 17 {
        return None;
    }

    let (tag_bytes, ciphertext) = data.split_at(16);

    // Constant-time HMAC tag verification
    let mut mac =
        HmacSha256::new_from_slice(token.as_bytes()).expect("HMAC-SHA256 accepts any key length");
    mac.update(ciphertext);
    let expected = mac.finalize().into_bytes();
    if !bool::from(tag_bytes.ct_eq(&expected[..16])) {
        return None;
    }

    // CTR decryption (same keystream)
    let enc_key: [u8; 32] = {
        let mut h = Sha256::new();
        h.update(token.as_bytes());
        h.update(b":enc");
        h.finalize().into()
    };

    let mut plaintext = Vec::with_capacity(ciphertext.len());
    for (block_idx, chunk) in ciphertext.chunks(32).enumerate() {
        let mut block_input = [0u8; 36];
        block_input[..32].copy_from_slice(&enc_key);
        block_input[32..].copy_from_slice(&u32::try_from(block_idx).expect("REASON").to_le_bytes());
        let keystream: [u8; 32] = Sha256::digest(block_input).into();
        for (b, k) in chunk.iter().zip(keystream.iter()) {
            plaintext.push(b ^ k);
        }
    }

    String::from_utf8(plaintext).ok()
}

/// Checks if a token is valid without consuming it (to display form).
pub fn peek(token: &str) -> bool {
    let store = TOKENS.lock().unwrap();
    let now = Instant::now();
    store.get(token).is_some_and(|v| v.expires_at > now)
}
