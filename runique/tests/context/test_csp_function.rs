//! Tests — context/tera/csp.rs
//! Couvre : nonce_function (retourne le nonce CSP ou une chaîne vide)

use runique::context::nonce_function;
use serde_json::Value;
use std::collections::HashMap;

// ── Helpers ──────────────────────────────────────────────────────────────────

fn args_nonce(nonce: &str) -> HashMap<String, Value> {
    let mut map = HashMap::new();
    map.insert("csp_nonce".to_string(), Value::String(nonce.to_string()));
    map
}

// ═══════════════════════════════════════════════════════════════
// Chemin nominal — nonce présent et valide
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_nonce_valeur_valide_retournee() {
    let args = args_nonce("abc123xyz");
    let result = nonce_function(&args).unwrap();
    assert_eq!(result, Value::String("abc123xyz".to_string()));
}

#[test]
fn test_nonce_valeur_longue() {
    let nonce = "a".repeat(64);
    let args = args_nonce(&nonce);
    let result = nonce_function(&args).unwrap();
    assert_eq!(result, Value::String(nonce));
}

#[test]
fn test_nonce_valeur_vide_retourne_vide() {
    let args = args_nonce("");
    let result = nonce_function(&args).unwrap();
    assert_eq!(result, Value::String(String::new()));
}

// ═══════════════════════════════════════════════════════════════
// Chemin dégradé — clé absente ou valeur invalide
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_nonce_sans_cle_retourne_vide() {
    let args: HashMap<String, Value> = HashMap::new();
    let result = nonce_function(&args).unwrap();
    assert_eq!(result, Value::String(String::new()));
}

#[test]
fn test_nonce_mauvaise_cle_retourne_vide() {
    let mut args = HashMap::new();
    args.insert("nonce".to_string(), Value::String("secret".to_string()));
    let result = nonce_function(&args).unwrap();
    assert_eq!(result, Value::String(String::new()));
}

#[test]
fn test_nonce_valeur_nombre_retourne_vide() {
    let mut args = HashMap::new();
    args.insert(
        "csp_nonce".to_string(),
        Value::Number(serde_json::Number::from(42)),
    );
    let result = nonce_function(&args).unwrap();
    assert_eq!(result, Value::String(String::new()));
}

#[test]
fn test_nonce_valeur_bool_retourne_vide() {
    let mut args = HashMap::new();
    args.insert("csp_nonce".to_string(), Value::Bool(true));
    let result = nonce_function(&args).unwrap();
    assert_eq!(result, Value::String(String::new()));
}

#[test]
fn test_nonce_valeur_null_retourne_vide() {
    let mut args = HashMap::new();
    args.insert("csp_nonce".to_string(), Value::Null);
    let result = nonce_function(&args).unwrap();
    assert_eq!(result, Value::String(String::new()));
}
