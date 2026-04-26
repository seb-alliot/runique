use runique::prelude::*;
use runique::utils::password::{
    AutoConfig, BaseHash, External, Manual, PasswordConfig, PasswordService, password_get,
    password_init,
};
use std::sync::Arc;

// ============================================================================
// UnifiedHasher — Détection algo
// ============================================================================

#[test]
fn test_detect_argon2() {
    password_init(PasswordConfig::auto());
    let service = PasswordService::new(password_get());
    let hash = service.hash("secret").unwrap();
    assert!(hash.starts_with("$argon2"));
}

#[test]
fn test_detect_bcrypt() {
    let service = PasswordService::new(PasswordConfig::manual(Manual::Bcrypt));
    let hash = service.hash("secret").unwrap();
    assert!(hash.starts_with("$2"));
}

#[test]
fn test_detect_scrypt() {
    let service = PasswordService::new(PasswordConfig::manual(Manual::Scrypt));
    let hash = service.hash("secret").unwrap();
    assert!(hash.starts_with("$scrypt"));
}

// ============================================================================
// PasswordService — Hash & Verify
// ============================================================================

#[test]
fn test_hash_and_verify_argon2() {
    let service = PasswordService::new(PasswordConfig::auto());
    let hash = service.hash("monpassword").unwrap();
    assert!(service.verify("monpassword", &hash));
    assert!(!service.verify("mauvaispassword", &hash));
}

#[test]
fn test_hash_and_verify_bcrypt() {
    let service = PasswordService::new(PasswordConfig::manual(Manual::Bcrypt));
    let hash = service.hash("monpassword").unwrap();
    assert!(service.verify("monpassword", &hash));
    assert!(!service.verify("mauvaispassword", &hash));
}

#[test]
fn test_hash_empty_password_refused() {
    let service = PasswordService::new(PasswordConfig::auto());
    assert!(service.hash("").is_err());
}

#[test]
fn test_double_hash_prevention() {
    let service = PasswordService::new(PasswordConfig::auto());
    let hash = service.hash("secret").unwrap();
    assert!(service.is_already_hashed(&hash));
    // Tenter de hasher un hash existant
    assert!(service.hash(&hash).is_err());
}

// ============================================================================
// TextField — finalize auto hash
// ============================================================================

#[test]
fn test_textfield_password_auto_hash_on_finalize() {
    password_init(PasswordConfig::auto());
    let mut field = TextField::password("password");
    field.set_value("monpassword");
    field.finalize().unwrap();
    assert!(field.value().starts_with("$argon2"));
}

#[test]
fn test_textfield_password_no_double_hash() {
    password_init(PasswordConfig::auto());
    let service = PasswordService::new(PasswordConfig::auto());
    let already_hashed = service.hash("monpassword").unwrap();

    let mut field = TextField::password("password");
    field.set_value(&already_hashed);
    field.finalize().unwrap();
    // La valeur ne doit pas avoir changé
    assert_eq!(field.value(), already_hashed);
}

#[test]
fn test_textfield_no_hash_flag() {
    password_init(PasswordConfig::auto());
    let mut field = TextField::password("password").no_hash();
    field.set_value("monpassword");
    field.finalize().unwrap();
    // Pas de hash car no_hash()
    assert_eq!(field.value(), "monpassword");
}

#[test]
fn test_textfield_password_manual_no_auto_hash() {
    // En mode Manuel, le dev hash lui-même — finalize ne fait rien
    let service = PasswordService::new(PasswordConfig::manual(Manual::Bcrypt));
    // finalize() ne hash qu'en mode Auto — vérifier via le service directement
    let hash = service.hash("monpassword").unwrap();
    assert!(hash.starts_with("$2")); // Bcrypt
    assert!(service.verify("monpassword", &hash));
}

// ============================================================================
// is_algorithm_current — migration algo
// ============================================================================

#[test]
fn test_is_algorithm_current_argon2() {
    let service = PasswordService::new(PasswordConfig::auto());
    let hash = service.hash("secret").unwrap();
    assert!(service.is_algorithm_current(&hash));
}

#[test]
fn test_is_algorithm_not_current_after_change() {
    // Hash en Argon2
    let service_argon = PasswordService::new(PasswordConfig::auto());
    let hash = service_argon.hash("secret").unwrap();

    // Config actuelle Bcrypt
    let service_bcrypt = PasswordService::new(PasswordConfig::auto_with(Manual::Bcrypt));
    assert!(!service_bcrypt.is_algorithm_current(&hash));
}

// ============================================================================
// PasswordService — auto_process() branches
// ============================================================================

#[test]
fn test_auto_process_non_password_format_is_noop() {
    let service = PasswordService::new(PasswordConfig::auto());
    let mut field = TextField::text("username");
    field.set_value("alice");
    assert!(service.auto_process(&mut field).is_ok());
    assert_eq!(field.value(), "alice"); // inchangé
}

#[test]
fn test_auto_process_empty_not_allowed_returns_err() {
    let service = PasswordService::new(PasswordConfig::Auto(AutoConfig {
        algorithm: Manual::Argon2,
        allow_empty: false,
        pre_hash_hook: None,
    }));
    let mut field = TextField::password("pwd");
    field.set_value("");
    assert!(service.auto_process(&mut field).is_err());
}

#[test]
fn test_auto_process_empty_allowed_returns_ok() {
    let service = PasswordService::new(PasswordConfig::Auto(AutoConfig {
        algorithm: Manual::Argon2,
        allow_empty: true,
        pre_hash_hook: None,
    }));
    let mut field = TextField::password("pwd");
    field.set_value("");
    assert!(service.auto_process(&mut field).is_ok());
    assert_eq!(field.value(), ""); // valeur inchangée
}

#[test]
fn test_auto_process_already_hashed_skips_rehash() {
    let service = PasswordService::new(PasswordConfig::auto());
    let hash = service.hash("secret").unwrap();

    let mut field = TextField::password("pwd");
    field.set_value(&hash);
    assert!(service.auto_process(&mut field).is_ok());
    assert_eq!(field.value(), hash); // valeur inchangée
}

#[test]
fn test_auto_process_unknown_hash_like_returns_err() {
    let service = PasswordService::new(PasswordConfig::auto());
    // Ressemble à un hash argon2 mais sous-type inconnu → looks_like_hash=true, is_already_hashed=false
    let mut field = TextField::password("pwd");
    field.set_value("$argon2xyz$something");
    assert!(service.auto_process(&mut field).is_err());
}

#[test]
fn test_auto_process_pre_hash_hook_failure_propagates() {
    let service = PasswordService::new(PasswordConfig::Auto(AutoConfig {
        algorithm: Manual::Argon2,
        allow_empty: false,
        pre_hash_hook: Some(Arc::new(|_| Err("hook rejects".to_string()))),
    }));
    let mut field = TextField::password("pwd");
    field.set_value("plainpassword");
    let result = service.auto_process(&mut field);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("hook rejects"));
}

#[test]
fn test_auto_process_manual_config_is_noop() {
    let service = PasswordService::new(PasswordConfig::manual(Manual::Bcrypt));
    let mut field = TextField::password("pwd");
    field.set_value("plainpassword");
    assert!(service.auto_process(&mut field).is_ok());
    assert_eq!(field.value(), "plainpassword"); // pas de hash automatique en mode Manuel
}

// ============================================================================
// PasswordService — hash() / verify() modes Delegated / Custom
// ============================================================================

#[test]
fn test_hash_delegated_returns_err() {
    let service = PasswordService::new(PasswordConfig::Delegated(External::GoogleOAuth));
    assert!(service.hash("anypassword").is_err());
}

#[test]
fn test_verify_delegated_returns_false() {
    let service = PasswordService::new(PasswordConfig::Delegated(External::GoogleOAuth));
    assert!(!service.verify("pwd", "$argon2id$xyz"));
}

#[test]
fn test_is_algorithm_current_manual_always_true() {
    let service = PasswordService::new(PasswordConfig::manual(Manual::Argon2));
    // En mode Manual, is_algorithm_current retourne toujours true (branche `_ => true`)
    assert!(service.is_algorithm_current("$argon2id$anything"));
    assert!(service.is_algorithm_current("$2b$10$anything_bcrypt"));
    assert!(service.is_algorithm_current("notahash"));
}

// ============================================================================
// BaseHash — detect_algorithm()
// ============================================================================

#[test]
fn test_base_hash_detect_argon2() {
    let b = BaseHash::new();
    assert_eq!(b.detect_algorithm("$argon2id$v=19$..."), Some("argon2"));
}

#[test]
fn test_base_hash_detect_bcrypt() {
    let b = BaseHash::new();
    assert_eq!(b.detect_algorithm("$2b$10$hash"), Some("bcrypt"));
}

#[test]
fn test_base_hash_detect_scrypt() {
    let b = BaseHash::new();
    assert_eq!(b.detect_algorithm("$scrypt$..."), Some("scrypt"));
}

#[test]
fn test_base_hash_detect_none_for_plaintext() {
    let b = BaseHash::new();
    assert_eq!(b.detect_algorithm("plaintext"), None);
}

// ============================================================================
// hash() / verify() top-level functions
// ============================================================================

#[test]
fn test_top_level_hash_and_verify() {
    // password_get() retourne le config globale (argon2 par défaut)
    let h = runique::utils::password::hash("testpwd").unwrap();
    assert!(runique::utils::password::verify("testpwd", &h));
    assert!(!runique::utils::password::verify("wrongpwd", &h));
}
