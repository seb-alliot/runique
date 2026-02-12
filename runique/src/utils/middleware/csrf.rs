use base64::{engine::general_purpose, Engine as _};
use hmac::{Hmac, Mac};
use rand::RngExt;
use serde::{Deserialize, Serialize};
use sha2::Sha256;

type HmacSha256 = Hmac<Sha256>;

/// Contexte pour la génération du token
pub enum CsrfContext<'a> {
    Anonymous { session_id: &'a str },
    Authenticated { user_id: i32 },
}

/// Token CSRF typé
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CsrfToken(pub String);


impl CsrfToken {
    /// Génération selon le contexte
    pub fn generate_with_context(ctx: CsrfContext, secret: &str) -> Self {
        let raw = match ctx {
            CsrfContext::Anonymous { session_id } => generation_token(secret, session_id),
            CsrfContext::Authenticated { user_id } => {
                generation_user_token(secret, &user_id.to_string())
            }
        };
        CsrfToken(raw)
    }
    /// Masque le token pour l’injection
    pub fn masked(&self) -> Self {
        CsrfToken(mask_csrf_token(&self.0))
    }

    /// Démasque le token
    pub fn unmasked(masked: &str) -> Result<Self, &'static str> {
        let unmasked_hex = unmask_csrf_token(masked)?;
        Ok(CsrfToken(unmasked_hex))
    }

    /// Accès au token brut
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Génération HMAC-SHA256 pour utilisateur anonyme
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

/// Génération HMAC-SHA256 pour utilisateur connecté
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

/// Masquage pour protection BREACH
pub fn mask_csrf_token(token_hex: &str) -> String {
    // Remplacer expect par une gestion d'erreur ou un fallback
    let token_bytes = hex::decode(token_hex).expect("Invalid hex token");

    let mut rng = rand::rng();

    let mask: Vec<u8> = (0..token_bytes.len()).map(|_| rng.random()).collect();

    // XOR du token avec le masque
    let masked: Vec<u8> = token_bytes
        .iter()
        .zip(mask.iter())
        .map(|(t, m)| t ^ m)
        .collect();

    // Concaténer masque + token_masqué
    let mut result = mask;
    result.extend(masked);

    // Encoder en base64
    general_purpose::STANDARD.encode(&result)
}

/// Démasquage
pub fn unmask_csrf_token(masked_token_b64: &str) -> Result<String, &'static str> {
    let decoded = general_purpose::STANDARD
        .decode(masked_token_b64)
        .map_err(|_| "Invalid base64")?;

    // Vérifier que la taille est paire (masque + token)
    if decoded.len() % 2 != 0 {
        return Err("Invalid token length");
    }

    let token_len = decoded.len() / 2;

    // Séparer masque et token masqué
    let (mask, masked) = decoded.split_at(token_len);

    // XOR inverse pour récupérer le token original
    let token_bytes: Vec<u8> = masked.iter().zip(mask.iter()).map(|(m, k)| m ^ k).collect();

    // Convertir en hex
    Ok(hex::encode(token_bytes))
}
