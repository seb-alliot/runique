//! Password reset tokens — HMAC-SHA256 email encryption + DB-backed, single-use,
//! hashed token store (table `eihwaz_reset_tokens`).
//!
//! The raw token lives only in the email link. The DB stores its **hash**, so a
//! DB read leak cannot be replayed. The reset target is the `user_id` carried by
//! the token row (server-derived) — never a URL/form field (IDOR-safe).
use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use hmac::{Hmac, KeyInit, Mac};
use sea_orm::{
    ActiveValue::Set, ColumnTrait, Condition, DatabaseConnection, DbErr, EntityTrait, QueryFilter,
};
use sha2::{Digest, Sha256};
use std::time::Duration;
use subtle::ConstantTimeEq;
use uuid::Uuid;

use crate::utils::pk::Pk;

mod entity;

type HmacSha256 = Hmac<Sha256>;

/// Hashes a raw token (the DB lookup key). The raw token is never stored.
fn hash_token(token: &str) -> String {
    URL_SAFE_NO_PAD.encode(Sha256::digest(token.as_bytes()))
}

/// Generates a single-use reset token for `user_id`, valid for `ttl`.
/// Persists the token **hash** + expiry; returns the **raw** token for the link.
///
/// Requesting a new token drops the user's previous tokens (a fresh link
/// invalidates older ones) and any globally expired row.
pub async fn generate(
    db: &DatabaseConnection,
    user_id: Pk,
    ttl: Duration,
) -> Result<String, DbErr> {
    let token = Uuid::new_v4().to_string();
    let now = chrono::Utc::now().naive_utc();
    let expires_at = chrono::Duration::from_std(ttl)
        .ok()
        .and_then(|d| now.checked_add_signed(d))
        .unwrap_or(now);

    // Best-effort cleanup; a failure here does not compromise the new token below.
    let _ = entity::Entity::delete_many()
        .filter(
            Condition::any()
                .add(entity::Column::UserId.eq(user_id))
                .add(entity::Column::ExpiresAt.lt(now)),
        )
        .exec(db)
        .await;

    let model = entity::ActiveModel {
        token_hash: Set(hash_token(&token)),
        user_id: Set(user_id),
        expires_at: Set(expires_at),
        ..Default::default()
    };
    entity::Entity::insert(model).exec(db).await?;
    Ok(token)
}

/// Consumes a token (single use) → returns the bound `user_id` if valid.
///
/// The row is claimed by deleting it: under a concurrent double-submit both reads
/// may see the row, but only the delete that affects exactly one row wins.
pub async fn consume(db: &DatabaseConnection, token: &str) -> Option<Pk> {
    let now = chrono::Utc::now().naive_utc();
    let row = entity::Entity::find()
        .filter(entity::Column::TokenHash.eq(hash_token(token)))
        .filter(entity::Column::ExpiresAt.gt(now))
        .one(db)
        .await
        .ok()??;

    let deleted = entity::Entity::delete_by_id(row.id).exec(db).await.ok()?;
    (deleted.rows_affected == 1).then_some(row.user_id)
}

/// Checks a token's validity without consuming it (to display the reset form).
pub async fn peek(db: &DatabaseConnection, token: &str) -> bool {
    let now = chrono::Utc::now().naive_utc();
    entity::Entity::find()
        .filter(entity::Column::TokenHash.eq(hash_token(token)))
        .filter(entity::Column::ExpiresAt.gt(now))
        .one(db)
        .await
        .ok()
        .flatten()
        .is_some()
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
