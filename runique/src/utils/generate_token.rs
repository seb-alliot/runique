use base64::{engine::general_purpose, Engine as _};
use hmac::{Hmac, Mac};
use rand::Rng;
use sha2::Sha256;

type HmacSha256 = Hmac<Sha256>;

/// Génère un token CSRF avec HMAC-SHA256 + timestamp
pub fn generate_token(secret_key: &str, session_id: &str) -> String {
    let mut mac =
        HmacSha256::new_from_slice(secret_key.as_bytes()).expect("HMAC can take key of any size");

    mac.update(b"runique.middleware.csrf");
    mac.update(session_id.as_bytes());

    let timestamp: String = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos()
        .to_string();
    mac.update(timestamp.as_bytes());

    let result = mac.finalize();
    hex::encode(result.into_bytes())
}

pub fn generate_user_token(secret_key: &str, user_id: &str) -> String {
    let mut mac =
        HmacSha256::new_from_slice(secret_key.as_bytes()).expect("HMAC can take key of any size");

    mac.update(b"runique.middleware.user_token");
    mac.update(user_id.as_bytes());

    let timestamp: String = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos()
        .to_string();
    mac.update(timestamp.as_bytes());

    let result = mac.finalize();
    hex::encode(result.into_bytes())
}
/// Masque un token CSRF pour prévenir l'attaque BREACH
///
/// Prend un token hex et retourne un token masqué en base64
pub fn mask_csrf_token(token_hex: &str) -> String {
    // Convertir hex en bytes
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

/// Démasque un token CSRF
///
/// Prend un token masqué en base64 et retourne le token original en hex
pub fn unmask_csrf_token(masked_token_b64: &str) -> Result<String, &'static str> {
    // Décoder le base64
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
