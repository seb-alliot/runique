//! Password reset tokens — DB-backed, single-use, hashed token store
//! (table `eihwaz_reset_tokens`).
//!
//! The raw token lives only in the email link. The DB stores its **hash**, so a
//! DB read leak cannot be replayed. The reset target is the `user_id` carried by
//! the token row (server-derived) — never a URL/form field (IDOR-safe). The
//! typed email on the reset form is checked against that account in the
//! database; it is never carried in the link.
use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use sea_orm::{ActiveValue::Set, ColumnTrait, Condition, DbErr, EntityTrait, QueryFilter};
use sha2::{Digest, Sha256};
use std::time::Duration;
use uuid::Uuid;

use crate::utils::aliases::ADb;
use crate::utils::pk::Pk;

mod entity;

/// Hashes a raw token (the DB lookup key). The raw token is never stored.
fn hash_token(token: &str) -> String {
    URL_SAFE_NO_PAD.encode(Sha256::digest(token.as_bytes()))
}

/// Generates a single-use reset token for `user_id`, valid for `ttl`.
/// Persists the token **hash** + expiry; returns the **raw** token for the link.
///
/// Requesting a new token drops the user's previous tokens (a fresh link
/// invalidates older ones) and any globally expired row.
pub async fn generate(db: &ADb, user_id: Pk, ttl: Duration) -> Result<String, DbErr> {
    let token = Uuid::new_v4().to_string();
    let now = chrono::Utc::now().naive_utc();
    let expires_at = chrono::Duration::from_std(ttl)
        .ok()
        .and_then(|d| now.checked_add_signed(d))
        .unwrap_or(now);

    // Best-effort cleanup; a failure here does not compromise the new token below,
    // mais on le logge (tokens périmés qui s'accumulent) plutôt que de l'avaler.
    if let Err(e) = entity::Entity::delete_many()
        .filter(
            Condition::any()
                .add(entity::Column::UserId.eq(user_id))
                .add(entity::Column::ExpiresAt.lt(now)),
        )
        .exec(db)
        .await
    {
        tracing::warn!(user_id = %user_id, error = %e, "reset token cleanup failed");
    }

    let model = entity::ActiveModel {
        token_hash: Set(hash_token(&token)),
        user_id: Set(user_id),
        expires_at: Set(expires_at),
        ..Default::default()
    };
    entity::Entity::insert(model).exec(db).await?;
    Ok(token)
}

/// Returns the bound `user_id` if the token is still valid, without consuming it.
pub async fn resolve(db: &ADb, token: &str) -> Option<Pk> {
    let now = chrono::Utc::now().naive_utc();
    entity::Entity::find()
        .filter(entity::Column::TokenHash.eq(hash_token(token)))
        .filter(entity::Column::ExpiresAt.gt(now))
        .one(db)
        .await
        .ok()
        .flatten()
        .map(|row| row.user_id)
}

/// Consumes a token (single use) → returns the bound `user_id` if valid.
///
/// The row is claimed by deleting it: under a concurrent double-submit both reads
/// may see the row, but only the delete that affects exactly one row wins.
pub async fn consume(db: &ADb, token: &str) -> Option<Pk> {
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
pub async fn peek(db: &ADb, token: &str) -> bool {
    resolve(db, token).await.is_some()
}
