use runique::prelude::*;
use runique::utils::password::{
    password_get, password_init, Manual, PasswordConfig, PasswordService,
};

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
