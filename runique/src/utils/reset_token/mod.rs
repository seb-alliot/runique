use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use sha2::{Digest, Sha256};
use std::{
    collections::HashMap,
    sync::{LazyLock, Mutex},
    time::{Duration, Instant},
};
use uuid::Uuid;

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

/// Chiffre l'email avec le token comme clé (XOR + SHA256 keystream).
/// Résultat : base64url, sûr dans une URL.
pub fn encrypt_email(token: &str, email: &str) -> String {
    let key = Sha256::digest(token.as_bytes());
    let encrypted: Vec<u8> = email
        .as_bytes()
        .iter()
        .enumerate()
        .map(|(i, b)| b ^ key[i % 32])
        .collect();
    URL_SAFE_NO_PAD.encode(&encrypted)
}

/// Déchiffre l'email depuis une valeur base64url produite par `encrypt_email`.
/// Retourne `None` si le décodage échoue.
pub fn decrypt_email(token: &str, encoded: &str) -> Option<String> {
    let encrypted = URL_SAFE_NO_PAD.decode(encoded).ok()?;
    let key = Sha256::digest(token.as_bytes());
    let decrypted: Vec<u8> = encrypted
        .iter()
        .enumerate()
        .map(|(i, b)| b ^ key[i % 32])
        .collect();
    String::from_utf8(decrypted).ok()
}

/// Vérifie qu'un token est valide sans le consommer (pour afficher le formulaire).
pub fn peek(token: &str) -> bool {
    let store = TOKENS.lock().unwrap();
    let now = Instant::now();
    store.get(token).is_some_and(|v| v.expires_at > now)
}
