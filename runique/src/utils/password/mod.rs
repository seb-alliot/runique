//! Password hashing and verification — Argon2, bcrypt, scrypt via `PasswordHasher` trait.

// === password/hasher.rs ===
// argon2 et scrypt sont désormais tous deux sur password-hash 0.6 — un seul
// jeu d'imports partagé (contrairement à avant : scrypt était resté sur 0.5,
// une version incompatible malgré le nom identique des traits/types).
use argon2::{
    Argon2,
    password_hash::{PasswordHasher as _, PasswordVerifier as _, phc::PasswordHash},
};
use bcrypt::{DEFAULT_COST, hash as bcrypt_hash, verify as bcrypt_verify};
use scrypt::Scrypt;

use crate::utils::trad::{t, tf};
use dyn_clone::DynClone;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

pub trait PasswordHasher: Send + Sync + DynClone + Debug {
    fn hash(&self, password: &str) -> Result<String, String>;
    fn verify(&self, password: &str, hash: &str) -> bool;
    fn algorithm_name(&self) -> &str;
}

impl Clone for Box<dyn PasswordHasher> {
    fn clone(&self) -> Box<dyn PasswordHasher> {
        dyn_clone::clone_box(&**self)
    }
}

#[derive(Debug, Clone)]
pub struct BaseHash;

/// Runs `verify_once(hash)`. If it errors — the hash doesn't even parse — re-runs it
/// against `dummy` to burn the same computation time before returning `false`: a
/// malformed stored hash must not answer faster than a well-formed one with the wrong
/// password, or timing leaks which stored hashes are well-formed (same principle as
/// `dummy_hash()` for user enumeration, applied to the parse step of each algorithm).
fn verify_constant_time<E>(
    verify_once: impl Fn(&str) -> Result<bool, E>,
    hash: &str,
    dummy: &str,
) -> bool {
    match verify_once(hash) {
        Ok(matched) => matched,
        Err(_) => {
            let _ = verify_once(dummy);
            false
        }
    }
}

impl BaseHash {
    pub fn new() -> Self {
        Self
    }

    pub fn hash(&self, password: &str, algorithm: &Manual) -> Result<String, String> {
        match algorithm {
            Manual::Argon2 => self.hash_argon2(password),
            Manual::Bcrypt => self.hash_bcrypt(password),
            Manual::Scrypt => self.hash_scrypt(password),
            Manual::Custom(hasher) => hasher.hash(password),
        }
    }
    #[must_use]
    pub fn verify(&self, password: &str, hash: &str) -> bool {
        if hash.starts_with("$argon2") {
            self.verify_argon2(password, hash)
        } else if hash.starts_with("$2") {
            self.verify_bcrypt(password, hash)
        } else if hash.starts_with("$scrypt") {
            self.verify_scrypt(password, hash)
        } else {
            false
        }
    }
    #[must_use]
    pub fn detect_algorithm(&self, hash: &str) -> Option<&'static str> {
        if hash.starts_with("$argon2") {
            Some("argon2")
        } else if hash.starts_with("$2") {
            Some("bcrypt")
        } else if hash.starts_with("$scrypt") {
            Some("scrypt")
        } else {
            None
        }
    }

    fn hash_argon2(&self, password: &str) -> Result<String, String> {
        if password.is_empty() {
            return Err(t("forms.password_empty").into_owned());
        }
        // argon2 0.6 : `hash_password` génère le sel en interne (via `getrandom`),
        // il ne prend plus de `SaltString`/RNG explicite en paramètre.
        Argon2::default()
            .hash_password(password.as_bytes())
            .map(|h| h.to_string())
            .map_err(|e| tf("forms.hash_error", &[&e.to_string()]).clone())
    }

    fn verify_argon2(&self, password: &str, hash: &str) -> bool {
        verify_constant_time(
            |h| {
                PasswordHash::new(h)
                    .map(|p| Argon2::default().verify_password(password.as_bytes(), &p).is_ok())
            },
            hash,
            dummy_hash(),
        )
    }

    fn hash_bcrypt(&self, password: &str) -> Result<String, String> {
        if password.is_empty() {
            return Err(t("forms.password_empty").into_owned());
        }
        bcrypt_hash(password, DEFAULT_COST)
            .map_err(|e| tf("forms.hash_error", &[&e.to_string()]).clone())
    }

    fn verify_bcrypt(&self, password: &str, hash: &str) -> bool {
        // `bcrypt::verify` ne renvoie `Err` que si le hash ne parse pas (`split_hash`) —
        // jamais pour "mot de passe faux", qui donne `Ok(false)`. Même traitement que
        // les autres algorithmes : brûler le même temps sur un hash mal formé.
        verify_constant_time(|h| bcrypt_verify(password, h), hash, &DUMMY_HASH_BCRYPT)
    }

    fn hash_scrypt(&self, password: &str) -> Result<String, String> {
        if password.is_empty() {
            return Err(t("forms.password_empty").into_owned());
        }
        // Comme argon2 0.6, `hash_password` génère le sel en interne (`getrandom`).
        // `Scrypt` 0.12 n'est plus un unit struct (champ `params` privé) — passer par `default()`.
        Scrypt::default()
            .hash_password(password.as_bytes())
            .map(|h| h.to_string())
            .map_err(|e| tf("forms.hash_error", &[&e.to_string()]).clone())
    }

    fn verify_scrypt(&self, password: &str, hash: &str) -> bool {
        verify_constant_time(
            |h| {
                PasswordHash::new(h)
                    .map(|p| Scrypt::default().verify_password(password.as_bytes(), &p).is_ok())
            },
            hash,
            &DUMMY_HASH_SCRYPT,
        )
    }
}

impl Default for BaseHash {
    fn default() -> Self {
        Self::new()
    }
}

// === password/config.rs ===
#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum PasswordConfig {
    Delegated(External),
    Auto(AutoConfig),
    Manual(Manual),
    #[serde(skip)]
    Custom(Box<dyn PasswordHandler>),
}

impl Default for PasswordConfig {
    fn default() -> Self {
        Self::Auto(AutoConfig::default())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum External {
    GoogleOAuth,
    Microsoft,
    Apple,
    Ldap(String),
    Saml(String),
    Custom {
        name: String,
        authorize_url: String,
        token_url: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Manual {
    Argon2,
    Bcrypt,
    Scrypt,
    #[serde(skip)]
    Custom(Box<dyn PasswordHasher>),
}

use std::sync::Arc;

type PreHashHook = dyn Fn(&str) -> Result<(), String> + Send + Sync;

#[derive(Serialize, Deserialize)]
pub struct AutoConfig {
    pub algorithm: Manual,
    pub allow_empty: bool,
    #[serde(skip)]
    pub pre_hash_hook: Option<Arc<PreHashHook>>,
}

// Manual implementation of Debug and Clone for AutoConfig
impl std::fmt::Debug for AutoConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AutoConfig")
            .field("algorithm", &self.algorithm)
            .field("allow_empty", &self.allow_empty)
            .field(
                "pre_hash_hook",
                &self.pre_hash_hook.as_ref().map(|_| "Some(Fn)"),
            )
            .finish()
    }
}

impl Clone for AutoConfig {
    fn clone(&self) -> Self {
        Self {
            algorithm: self.algorithm.clone(),
            allow_empty: self.allow_empty,
            pre_hash_hook: self.pre_hash_hook.clone(),
        }
    }
}

pub trait PasswordHandler: Send + Sync + DynClone {
    fn name(&self) -> &str;
    fn create_field(&self, name: &str) -> Box<dyn crate::forms::FormField>;
    fn validate_input(&self, input: &str) -> Result<(), String>;
    fn transform(&self, input: &str) -> Result<String, String>;
    fn verify(&self, input: &str, stored: &str) -> bool;
}

dyn_clone::clone_trait_object!(PasswordHandler);

impl Debug for dyn PasswordHandler {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PasswordHandler")
            .field("name", &self.name())
            .finish()
    }
}

impl Default for AutoConfig {
    fn default() -> Self {
        Self {
            algorithm: Manual::Argon2,
            allow_empty: false,
            pre_hash_hook: None,
        }
    }
}

impl PasswordConfig {
    pub fn auto() -> Self {
        Self::Auto(AutoConfig::default())
    }
    pub fn auto_with(algorithm: Manual) -> Self {
        Self::Auto(AutoConfig {
            algorithm,
            ..Default::default()
        })
    }
    pub fn manual(algorithm: Manual) -> Self {
        Self::Manual(algorithm)
    }
    pub fn oauth(provider: External) -> Self {
        Self::Delegated(provider)
    }

    pub fn custom<H: PasswordHandler + 'static>(handler: H) -> Self {
        Self::Custom(Box::new(handler))
    }
}

// === password/service.rs ===

use crate::forms::{SpecialFormat, TextField};

#[derive(Debug)]
pub struct PasswordService {
    config: PasswordConfig,
    hasher: BaseHash,
}

impl PasswordService {
    pub fn new(config: PasswordConfig) -> Self {
        Self {
            config,
            hasher: BaseHash::new(),
        }
    }

    pub fn auto_process(&self, field: &mut TextField) -> Result<(), String> {
        match &self.config {
            PasswordConfig::Auto(config) => {
                if field.format != SpecialFormat::Password {
                    return Ok(());
                }

                let value = &field.base.value;

                if value.is_empty() {
                    if !config.allow_empty {
                        return Err("Empty password not allowed".to_string());
                    }
                    return Ok(());
                }

                if self.is_already_hashed(value) {
                    return Ok(());
                }

                if self.looks_like_hash(value) {
                    return Err(format!(
                        "The field '{}' contains a value resembling an unrecognized hash",
                        field.base.name
                    ));
                }

                if let Some(hook) = &config.pre_hash_hook {
                    hook(value)?;
                }

                field.base.value = self.hasher.hash(value, &config.algorithm)?;
                Ok(())
            }
            _ => Ok(()),
        }
    }

    pub fn hash(&self, password: &str) -> Result<String, String> {
        // check allow_empty policy for Auto mode
        if password.is_empty() {
            match &self.config {
                PasswordConfig::Auto(config) if !config.allow_empty => {
                    return Err("Empty password not allowed".to_string());
                }
                _ => {}
            }
        }

        if self.looks_like_hash(password) {
            return Err("The password resembles an already processed hash".to_string());
        }

        match &self.config {
            PasswordConfig::Auto(config) => self.hasher.hash(password, &config.algorithm),
            PasswordConfig::Manual(algorithm) => self.hasher.hash(password, algorithm),
            PasswordConfig::Custom(handler) => handler.transform(password),
            PasswordConfig::Delegated(_) => Err("No hashing in delegated mode".to_string()),
        }
    }
    #[must_use]
    pub fn verify(&self, password: &str, hash: &str) -> bool {
        match &self.config {
            PasswordConfig::Auto(_) | PasswordConfig::Manual(_) => {
                self.hasher.verify(password, hash)
            }
            PasswordConfig::Custom(handler) => handler.verify(password, hash),
            PasswordConfig::Delegated(_) => false,
        }
    }
    #[must_use]
    pub fn is_algorithm_current(&self, hash: &str) -> bool {
        match &self.config {
            PasswordConfig::Auto(config) => {
                let detected = self.hasher.detect_algorithm(hash);
                matches!(
                    (&detected, &config.algorithm),
                    (Some("argon2"), Manual::Argon2)
                        | (Some("bcrypt"), Manual::Bcrypt)
                        | (Some("scrypt"), Manual::Scrypt)
                )
            }
            _ => true,
        }
    }
    #[must_use]
    pub fn is_already_hashed(&self, value: &str) -> bool {
        value.starts_with("$argon2id$")
            || value.starts_with("$argon2i$")
            || value.starts_with("$argon2d$")
            || value.starts_with("$2")
            || value.starts_with("$scrypt$")
    }

    fn looks_like_hash(&self, value: &str) -> bool {
        value.starts_with("$argon2") || value.starts_with("$2") || value.starts_with("$scrypt")
    }
}

use std::sync::OnceLock;

pub static PASSWORD_CONFIG: OnceLock<PasswordConfig> = OnceLock::new();

// Pre-computed dummy hash — used to run a full verify even when the user is not found,
// preventing user enumeration via timing differences (constant-time authentication flow).
static DUMMY_HASH: std::sync::LazyLock<String> = std::sync::LazyLock::new(|| {
    BaseHash::new()
        .hash("__runique_dummy__", &Manual::Argon2)
        .unwrap_or_else(|_| {
            "$argon2id$v=19$m=19456,t=2,p=1$c29tZXNhbHQ$RdescudvJCsgt3ub+b+dWRWJTmaaJObG"
                .to_string()
        })
});

/// Returns a pre-computed dummy hash for constant-time authentication.
/// Always use this when the user is not found — never short-circuit before `verify()`.
#[must_use]
pub fn dummy_hash() -> &'static str {
    &DUMMY_HASH
}

// Dummy hashes for bcrypt/scrypt — used only by `verify_constant_time` to burn the
// same computation time when a stored hash fails to parse for that algorithm.
// Unlike `dummy_hash()` (public, algorithm-agnostic, used for user-enumeration
// defense), these are private: each `verify_*` needs a dummy in its own format.
static DUMMY_HASH_BCRYPT: std::sync::LazyLock<String> = std::sync::LazyLock::new(|| {
    bcrypt_hash("__runique_dummy__", DEFAULT_COST)
        .expect("bcrypt hashing a fixed short string should never fail")
});

static DUMMY_HASH_SCRYPT: std::sync::LazyLock<String> = std::sync::LazyLock::new(|| {
    Scrypt::default()
        .hash_password(b"__runique_dummy__")
        .map(|h| h.to_string())
        .expect("scrypt hashing a fixed short string should never fail")
});

pub fn password_init(config: PasswordConfig) {
    if PASSWORD_CONFIG.set(config).is_err()
        && let Some(level) = crate::utils::runique_log::get_log()
            .auth
            .as_ref()
            .and_then(|a| a.password_init)
    {
        crate::runique_log!(
            level,
            "password_init() called multiple times — initial configuration is kept"
        );
    }
}

pub fn password_get() -> PasswordConfig {
    PASSWORD_CONFIG.get_or_init(PasswordConfig::auto).clone()
}

pub fn hash(password: &str) -> Result<String, String> {
    let svc = PasswordService::new(password_get());
    svc.hash(password)
}
#[must_use]
pub fn verify(password: &str, hash: &str) -> bool {
    let svc = PasswordService::new(password_get());
    svc.verify(password, hash)
}
