# Cours 4 : Utilitaires et Helpers

## 🎯 Objectif

Créer des fonctions utilitaires réutilisables : génération de tokens, hashing, etc.

## 🔧 Implémentations

### 1. Génération de Tokens Sécurisés

#### Principe

Un token doit être :
- **Unique** : Différent à chaque génération
- **Imprévisible** : Impossible à deviner
- **Vérifiable** : On peut vérifier qu'il est valide

#### Implémentation avec HMAC

```rust
use hmac::{Hmac, Mac};
use sha2::Sha256;

type HmacSha256 = Hmac<Sha256>;

pub fn generate_token(secret_key: &str, session_id: &str) -> String {
    // 1. Créer un MAC (Message Authentication Code)
    let mut mac = HmacSha256::new_from_slice(secret_key.as_bytes())
        .expect("HMAC can take key of any size");

    // 2. Ajouter un préfixe pour le contexte
    mac.update(b"runique.middleware.csrf");

    // 3. Ajouter l'ID de session (unique par utilisateur)
    mac.update(session_id.as_bytes());

    // 4. Ajouter un timestamp pour l'unicité
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos()
        .to_string();
    mac.update(timestamp.as_bytes());

    // 5. Finaliser et encoder en hex
    let result = mac.finalize();
    hex::encode(result.into_bytes())
}
```

**Pourquoi HMAC ?**
- **Cryptographiquement sûr** : Basé sur SHA-256
- **Déterministe** : Même entrée = même sortie (sans timestamp)
- **Rapide** : Efficace pour les tokens

#### Alternative : Tokens aléatoires

```rust
use rand::Rng;

pub fn generate_random_token() -> String {
    let mut rng = rand::thread_rng();
    let bytes: [u8; 32] = rng.gen();
    hex::encode(bytes)
}
```

**Quand utiliser ?**
- Tokens de session
- Tokens de réinitialisation de mot de passe
- Nonces CSP

### 2. Hashing de Mots de Passe

#### Principe

Ne JAMAIS stocker les mots de passe en clair. Utiliser Argon2 (ou bcrypt).

```rust
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHasher, SaltString},
    Argon2
};

pub fn hash_password(password: &str) -> Result<String, String> {
    // 1. Générer un salt aléatoire
    let salt = SaltString::generate(&mut OsRng);

    // 2. Créer l'instance Argon2
    let argon2 = Argon2::default();

    // 3. Hasher le mot de passe
    let password_hash = argon2
        .hash_password(password.as_bytes(), &salt)
        .map_err(|_| "Erreur lors du hachage".to_string())?
        .to_string();

    Ok(password_hash)
}

pub fn verify_password(password: &str, hash: &str) -> bool {
    use argon2::password_hash::PasswordVerifier;

    let parsed_hash = argon2::PasswordHash::new(hash)
        .ok()?;

    Argon2::default()
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok()
}
```

### 3. Validation d'Email

```rust
pub fn is_valid_email(email: &str) -> bool {
    // Validation basique
    email.contains('@')
        && email.contains('.')
        && email.len() > 5
        && !email.starts_with('@')
        && !email.ends_with('@')
}

// Version avec regex (plus stricte)
use regex::Regex;

pub fn is_valid_email_regex(email: &str) -> bool {
    lazy_static! {
        static ref EMAIL_RE: Regex = Regex::new(
            r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$"
        ).unwrap();
    }
    EMAIL_RE.is_match(email)
}
```

### 4. Masquage de Données Sensibles

```rust
pub fn mask_password(url: &str) -> String {
    // Masquer le mot de passe dans une URL
    // postgres://user:password@host/db
    // → postgres://user:***@host/db

    if let Some(at_pos) = url.find('@') {
        if let Some(colon_pos) = url[..at_pos].rfind(':') {
            let mut masked = url.to_string();
            let start = colon_pos + 1;
            let end = at_pos;
            masked.replace_range(start..end, "***");
            return masked;
        }
    }
    url.to_string()
}
```

### 5. Formatage de Durée

```rust
pub fn format_duration(seconds: u64) -> String {
    let hours = seconds / 3600;
    let minutes = (seconds % 3600) / 60;
    let secs = seconds % 60;

    if hours > 0 {
        format!("{}h {}m {}s", hours, minutes, secs)
    } else if minutes > 0 {
        format!("{}m {}s", minutes, secs)
    } else {
        format!("{}s", secs)
    }
}
```

## 🎓 Exercices

### Exercice 1 : Token avec expiration

Créez un système de tokens avec expiration :
```rust
struct Token {
    value: String,
    expires_at: SystemTime,
}

fn generate_token_with_expiry(ttl: Duration) -> Token {
    // ...
}
```

### Exercice 2 : Slug generation

Créez une fonction pour générer des slugs (URL-friendly) :
```rust
fn slugify(text: &str) -> String {
    // "Hello World!" → "hello-world"
}
```

### Exercice 3 : Validation de force de mot de passe

```rust
fn password_strength(password: &str) -> PasswordStrength {
    // Vérifier longueur, majuscules, chiffres, etc.
}
```

## 💡 Bonnes pratiques

1. **Sécurité d'abord** : Utilisez des algorithmes cryptographiques éprouvés
2. **Ne réinventez pas la roue** : Utilisez des crates comme `argon2`, `hmac`
3. **Gestion d'erreurs** : Retournez `Result` plutôt que de paniquer
4. **Documentation** : Documentez les fonctions publiques

## 🔗 Ressources

- [Argon2](https://github.com/RustCrypto/password-hashes)
- [HMAC](https://en.wikipedia.org/wiki/HMAC)
