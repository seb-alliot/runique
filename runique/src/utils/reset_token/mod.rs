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

/// Génère un token de réinitialisation lié à un email (valide 1h, usage unique).
pub fn generate(email: &str) -> String {
    let token = Uuid::new_v4().to_string();
    let mut store = TOKENS.lock().unwrap();
    let now = Instant::now();
    store.retain(|_, v| v.expires_at > now);
    store.insert(
        token.clone(),
        ResetEntry {
            email: email.to_string(),
            expires_at: now + TOKEN_TTL,
        },
    );
    token
}

/// Consomme un token (usage unique) → retourne l'email associé si valide.
pub fn consume(token: &str) -> Option<String> {
    let mut store = TOKENS.lock().unwrap();
    let now = Instant::now();
    if let Some(entry) = store.remove(token) {
        if entry.expires_at > now {
            return Some(entry.email);
        }
    }
    None
}

/// Chiffre l'email avec le token comme clé (CTR-SHA256 + HMAC-SHA256).
/// Résultat : base64url, sûr dans une URL.
///
/// Construction : enc_key = SHA256(token || ":enc"), keystream[i] = SHA256(enc_key || i_le32)
/// Authentification : HMAC-SHA256(token, ciphertext), tag tronqué à 16 octets en préfixe.
pub fn encrypt_email(token: &str, email: &str) -> String {
    let email_bytes = email.as_bytes();

    // Clé de chiffrement dérivée du token — unique par token (UUID4)
    let enc_key: [u8; 32] = {
        let mut h = Sha256::new();
        h.update(token.as_bytes());
        h.update(b":enc");
        h.finalize().into()
    };

    // Chiffrement CTR : keystream[i] = SHA256(enc_key || counter_le32)
    let mut ciphertext = Vec::with_capacity(email_bytes.len());
    for (block_idx, chunk) in email_bytes.chunks(32).enumerate() {
        let mut block_input = [0u8; 36];
        block_input[..32].copy_from_slice(&enc_key);
        block_input[32..].copy_from_slice(&(block_idx as u32).to_le_bytes());
        let keystream: [u8; 32] = Sha256::digest(block_input).into();
        for (b, k) in chunk.iter().zip(keystream.iter()) {
            ciphertext.push(b ^ k);
        }
    }

    // Tag d'authentification : HMAC-SHA256(token, ciphertext), tronqué à 16 octets
    let mut mac = HmacSha256::new_from_slice(token.as_bytes())
        .expect("HMAC-SHA256 accepte toute longueur de clé");
    mac.update(&ciphertext);
    let tag = mac.finalize().into_bytes();

    // Sortie : tag[0..16] || ciphertext (tag en préfixe pour rejet rapide)
    let mut output = Vec::with_capacity(16 + ciphertext.len());
    output.extend_from_slice(&tag[..16]);
    output.extend_from_slice(&ciphertext);

    URL_SAFE_NO_PAD.encode(&output)
}

/// Déchiffre l'email depuis une valeur base64url produite par `encrypt_email`.
/// Retourne `None` si le décodage, l'authentification ou l'UTF-8 échoue.
pub fn decrypt_email(token: &str, encoded: &str) -> Option<String> {
    let data = URL_SAFE_NO_PAD.decode(encoded).ok()?;
    // Minimum : 16 octets de tag + au moins 1 octet de ciphertext
    if data.len() < 17 {
        return None;
    }

    let (tag_bytes, ciphertext) = data.split_at(16);

    // Vérification du tag HMAC en temps constant
    let mut mac = HmacSha256::new_from_slice(token.as_bytes())
        .expect("HMAC-SHA256 accepte toute longueur de clé");
    mac.update(ciphertext);
    let expected = mac.finalize().into_bytes();
    if !bool::from(tag_bytes.ct_eq(&expected[..16])) {
        return None;
    }

    // Déchiffrement CTR (même keystream)
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
        block_input[32..].copy_from_slice(&(block_idx as u32).to_le_bytes());
        let keystream: [u8; 32] = Sha256::digest(block_input).into();
        for (b, k) in chunk.iter().zip(keystream.iter()) {
            plaintext.push(b ^ k);
        }
    }

    String::from_utf8(plaintext).ok()
}

/// Vérifie qu'un token est valide sans le consommer (pour afficher le formulaire).
pub fn peek(token: &str) -> bool {
    let store = TOKENS.lock().unwrap();
    let now = Instant::now();
    store.get(token).is_some_and(|v| v.expires_at > now)
}
