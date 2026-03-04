//! Tests — forms/prisme/csrf_gate.rs
//! Couvre : csrf_gate<T> — token valide (→ None) et invalide/manquant (→ Some avec erreur).

use reqwest::Method;
use runique::forms::field::RuniqueForm;
use runique::forms::prisme::csrf_gate;
use runique::middleware::LoginAdmin;
use std::collections::HashMap;
use std::sync::Arc;
use tera::Tera;

fn make_tera() -> Arc<Tera> {
    Arc::new(Tera::default())
}

// ═══════════════════════════════════════════════════════════════
// Token valide → Ok(None)
// ═══════════════════════════════════════════════════════════════

#[tokio::test]
async fn test_csrf_gate_token_valide_retourne_none() {
    let mut parsed: HashMap<String, Vec<String>> = HashMap::new();
    parsed.insert("csrf_token".to_string(), vec!["mon_token_csrf".to_string()]);

    let result =
        csrf_gate::<LoginAdmin>(&parsed, "mon_token_csrf", make_tera(), &Method::POST).await;
    assert!(result.is_ok());
    assert!(
        result.unwrap().is_none(),
        "Token valide doit retourner None"
    );
}

// ═══════════════════════════════════════════════════════════════
// Token manquant → Ok(Some(Prisme)) avec erreur CSRF
// ═══════════════════════════════════════════════════════════════

#[tokio::test]
async fn test_csrf_gate_token_manquant_retourne_some_avec_erreur() {
    let parsed: HashMap<String, Vec<String>> = HashMap::new(); // pas de csrf_token

    let result =
        csrf_gate::<LoginAdmin>(&parsed, "token_session", make_tera(), &Method::POST).await;
    assert!(result.is_ok());
    let maybe_prisme = result.unwrap();
    assert!(
        maybe_prisme.is_some(),
        "Token manquant doit retourner Some(Prisme)"
    );

    let prisme = maybe_prisme.unwrap();
    let form = prisme.0.get_form();
    // Le champ csrf_token doit avoir une erreur
    if let Some(csrf_field) = form.fields.get("csrf_token") {
        assert!(
            csrf_field.error().is_some(),
            "Le champ csrf_token doit avoir une erreur"
        );
        assert!(csrf_field
            .error()
            .unwrap()
            .contains("Invalid or missing CSRF token"));
    }
}

// ═══════════════════════════════════════════════════════════════
// Token invalide (mauvaise valeur) → Ok(Some(Prisme)) avec erreur
// ═══════════════════════════════════════════════════════════════

#[tokio::test]
async fn test_csrf_gate_token_incorrect_retourne_some() {
    let mut parsed: HashMap<String, Vec<String>> = HashMap::new();
    parsed.insert("csrf_token".to_string(), vec!["mauvais_token".to_string()]);

    let result = csrf_gate::<LoginAdmin>(&parsed, "bon_token", make_tera(), &Method::POST).await;
    assert!(result.is_ok());
    assert!(
        result.unwrap().is_some(),
        "Token incorrect doit retourner Some(Prisme)"
    );
}

// ═══════════════════════════════════════════════════════════════
// Token vide dans parsed → invalide
// ═══════════════════════════════════════════════════════════════

#[tokio::test]
async fn test_csrf_gate_token_vide_retourne_some() {
    let mut parsed: HashMap<String, Vec<String>> = HashMap::new();
    parsed.insert("csrf_token".to_string(), vec!["".to_string()]);

    let result =
        csrf_gate::<LoginAdmin>(&parsed, "token_session", make_tera(), &Method::POST).await;
    assert!(result.is_ok());
    assert!(
        result.unwrap().is_some(),
        "Token vide doit retourner Some(Prisme)"
    );
}

// ═══════════════════════════════════════════════════════════════
// Vec vide pour csrf_token → invalide
// ═══════════════════════════════════════════════════════════════

#[tokio::test]
async fn test_csrf_gate_vec_vide_retourne_some() {
    let mut parsed: HashMap<String, Vec<String>> = HashMap::new();
    parsed.insert("csrf_token".to_string(), vec![]);

    let result =
        csrf_gate::<LoginAdmin>(&parsed, "token_session", make_tera(), &Method::POST).await;
    assert!(result.is_ok());
    assert!(
        result.unwrap().is_some(),
        "Vec vide pour csrf_token doit retourner Some(Prisme)"
    );
}
