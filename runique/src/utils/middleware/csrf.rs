//! Génération et validation des tokens CSRF — HMAC-SHA256, masquage base64, contextes anonyme/authentifié.
use base64::{Engine as _, engine::general_purpose};
use hmac::{Hmac, Mac};
use rand::RngExt;
use serde::{Deserialize, Serialize};
use sha2::Sha256;

type HmacSha256 = Hmac<Sha256>;

/// Context for token generation
pub enum CsrfContext<'a> {
    Anonymous { session_id: &'a str },
    Authenticated { user_id: crate::utils::pk::Pk },
}

/// Typed CSRF token
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CsrfToken(pub String);

impl CsrfToken {
    /// Generation according to context
    #[must_use]
    pub fn generate_with_context(ctx: &CsrfContext, secret: &str) -> Self {
        let raw = match ctx {
            CsrfContext::Anonymous { session_id } => generation_token(secret, session_id),
            CsrfContext::Authenticated { user_id } => {
                generation_user_token(secret, &user_id.to_string())
            }
        };
        CsrfToken(raw)
    }
    /// Masks the token for injection
    pub fn masked(&self) -> Result<Self, &'static str> {
        mask_csrf_token(&self.0).map(CsrfToken)
    }

    /// Unmasks the token
    pub fn unmasked(masked: &str) -> Result<Self, &'static str> {
        let unmasked_hex = unmask_csrf_token(masked)?;
        Ok(CsrfToken(unmasked_hex))
    }

    /// Access to the raw token
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// HMAC-SHA256 generation for anonymous user
#[must_use]
pub fn generation_token(secret_key: &str, option: &str) -> String {
    let mut mac =
        HmacSha256::new_from_slice(secret_key.as_bytes()).expect("HMAC can take key of any size");

    mac.update(b"runique.middleware.csrf");
    mac.update(option.as_bytes());

    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos()
        .to_string();
    mac.update(timestamp.as_bytes());

    hex::encode(mac.finalize().into_bytes())
}

/// HMAC-SHA256 generation for authenticated user
#[must_use]
pub fn generation_user_token(secret_key: &str, option: &str) -> String {
    let mut mac =
        HmacSha256::new_from_slice(secret_key.as_bytes()).expect("HMAC can take key of any size");

    mac.update(b"runique.middleware.csrf");
    mac.update(option.as_bytes());

    let timestamp: String = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos()
        .to_string();
    mac.update(timestamp.as_bytes());

    let result = mac.finalize();
    hex::encode(result.into_bytes())
}

/// Masking for BREACH protection
pub fn mask_csrf_token(token_hex: &str) -> Result<String, &'static str> {
    let token_bytes = hex::decode(token_hex).map_err(|_| "Invalid hex token")?;

    let mut rng = rand::rng();

    let mask: Vec<u8> = (0..token_bytes.len()).map(|_| rng.random::<u8>()).collect();

    // XOR the token with the mask
    let masked: Vec<u8> = token_bytes
        .iter()
        .zip(mask.iter())
        .map(|(t, m)| t ^ m)
        .collect();

    // Concatenate mask + masked_token
    let mut result = mask;
    result.extend(masked);

    // Encode in base64 URL-safe (évite + et / qui sont corrompus par les forms URL-encoded)
    Ok(general_purpose::URL_SAFE_NO_PAD.encode(&result))
}

/// Unmasking
pub fn unmask_csrf_token(masked_token_b64: &str) -> Result<String, &'static str> {
    let decoded = general_purpose::URL_SAFE_NO_PAD
        .decode(masked_token_b64)
        .map_err(|_| "Invalid base64")?;

    // Check that the size is even (mask + token)
    if decoded.len() % 2 != 0 {
        return Err("Invalid token length");
    }

    let token_len = decoded.len() / 2;

    // Split mask and masked token
    let (mask, masked) = decoded.split_at(token_len);

    // Inverse XOR to retrieve the original token
    let token_bytes: Vec<u8> = masked.iter().zip(mask.iter()).map(|(m, k)| m ^ k).collect();

    // Convert to hex
    Ok(hex::encode(token_bytes))
}
